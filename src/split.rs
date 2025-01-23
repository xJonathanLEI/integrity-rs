use starknet_core::types::Felt;
use starknet_crypto::poseidon_hash_many;
use swiftness::{
    commit::stark_commit,
    oods::{eval_oods_boundary_poly_at_points, OodsEvaluationInfo},
    queries::{generate_queries, queries_to_points},
    types::{StarkCommitment, StarkWitness},
};
use swiftness_air::{
    domains::StarkDomains,
    layout::{GenericLayoutTrait, LayoutTrait},
    public_memory::PublicInput,
};
use swiftness_fri::{
    first_layer::gather_first_layer_queries,
    group::get_fri_group,
    layer::{
        compute_next_layer, FriLayerComputationParams, FriLayerQuery as SwiftnessFriLayerQuery,
    },
    types::LayerWitness,
};
use swiftness_stark::types::StarkProof;
use swiftness_transcript::transcript::Transcript;

use crate::{
    bindings::{
        FriLayerQuery, FriLayerWitness, FriVerificationStateConstant, FriVerificationStateVariable,
        StarkProofWithSerde, TableCommitment, TableCommitmentConfig, TableCommitmentWitness,
        VectorCommitment, VectorCommitmentConfig, VectorCommitmentWitness, VerifierConfiguration,
        VerifyProofFinalAndRegisterFactCall, VerifyProofInitialCall, VerifyProofStepCall,
    },
    IntegrityCalls,
};

/// A split START proof that can be used to generate Starknet function calls to the `integrity`
/// verifier contract.
#[derive(Debug)]
pub struct SplitProof {
    /// STARK proof with `fri_witness` stripped out.
    pub proof: StarkProofWithSerde,
    /// The state constants used throughout all verification steps.
    pub state_const: FriVerificationStateConstant,
    /// An iterator that returns intermediate and final steps.
    pub step_iter: VerifyProofStepParamIter,
}

/// An iterator that produces data necessary for constructing the intermediate and final
/// verification steps.
#[derive(Debug)]
pub struct VerifyProofStepParamIter {
    next_index: usize,
    n_layers: usize,
    next_queries: Vec<SwiftnessFriLayerQuery>,
    fri_group: Vec<Felt>,
    layer_witness: Vec<LayerWitness>,
    eval_points: Vec<Felt>,
    step_sizes: Vec<Felt>,
    last_layer_coefficients: Vec<Felt>,
}

impl SplitProof {
    /// Transforms the split proofs into `integrity` contract binding types by supplying a unique
    /// job ID and verifier configuration.
    pub fn into_calls(
        mut self,
        job_id: Felt,
        verifier_config: VerifierConfiguration,
    ) -> IntegrityCalls {
        IntegrityCalls {
            initial: VerifyProofInitialCall {
                job_id,
                verifier_config,
                stark_proof: self.proof,
            },
            intermediate_steps: self
                .step_iter
                .by_ref()
                .map(|(state_var, witness)| VerifyProofStepCall {
                    job_id,
                    state_constant: self.state_const.clone(),
                    state_variable: state_var,
                    witness,
                })
                .collect(),
            final_step: {
                let (state_var, witness) = self.step_iter.final_step();
                VerifyProofFinalAndRegisterFactCall {
                    job_id,
                    state_constant: self.state_const,
                    state_variable: state_var,
                    last_layer_coefficients: witness,
                }
            },
        }
    }
}

impl VerifyProofStepParamIter {
    /// Generates the final step after the iterator has been exhausted.
    ///
    /// The function panics if it's call before the iterator is exhausted. Make sure `.next()`
    /// returns [`None`] before calling.
    pub fn final_step(self) -> (FriVerificationStateVariable, Vec<Felt>) {
        if self.next_index != self.n_layers {
            panic!("`final_step` can only be used when the iterator has been exhausted")
        }

        (self.state_variable(), self.last_layer_coefficients)
    }

    fn state_variable(&self) -> FriVerificationStateVariable {
        FriVerificationStateVariable {
            iter: self.next_index.try_into().unwrap(),
            queries: self
                .next_queries
                .iter()
                .map(|query| FriLayerQuery {
                    index: query.index,
                    y_value: query.y_value,
                    x_inv_value: query.x_inv_value,
                })
                .collect(),
        }
    }
}

impl Iterator for VerifyProofStepParamIter {
    type Item = (FriVerificationStateVariable, FriLayerWitness);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.n_layers {
            // Intermediate layers exhausted. Call `.final_step()` to retrieve the final step
            // instead.
            None
        } else {
            let target_layer_witness = self.layer_witness.get(self.next_index).unwrap();

            // Params.
            let coset_size = Felt::TWO.pow_felt(self.step_sizes.get(self.next_index).unwrap());
            let params = FriLayerComputationParams {
                coset_size,
                fri_group: self.fri_group.clone(),
                eval_point: *self.eval_points.get(self.next_index).unwrap(),
            };

            let state_var = self.state_variable();
            let witness = FriLayerWitness {
                leaves: target_layer_witness.leaves.clone(),
                table_witness: TableCommitmentWitness {
                    vector: VectorCommitmentWitness {
                        authentications: target_layer_witness
                            .table_witness
                            .vector
                            .authentications
                            .clone(),
                    },
                },
            };

            let (next_queries, _, _) = compute_next_layer(
                &mut self.next_queries,
                &mut target_layer_witness.leaves.to_owned(),
                params,
            )
            .unwrap();

            self.next_index += 1;
            self.next_queries = next_queries;

            Some((state_var, witness))
        }
    }
}

/// Splits a [`StarkProof`] into a multi-step verification process.
///
/// This function does *not* verify the proof.
pub fn split_proof<Layout: GenericLayoutTrait + LayoutTrait>(
    proof: StarkProof,
) -> Result<SplitProof, swiftness_stark::stark::Error> {
    let n_original_columns = Layout::get_num_columns_first(&proof.public_input)
        .ok_or(swiftness_stark::stark::Error::ColumnMissing)?;
    let n_interaction_columns = Layout::get_num_columns_second(&proof.public_input)
        .ok_or(swiftness_stark::stark::Error::ColumnMissing)?;

    // Validate the public input.
    let stark_domains = StarkDomains::new(
        proof.config.log_trace_domain_size,
        proof.config.log_n_cosets,
    );

    // Compute the initial hash seed for the Fiat-Shamir transcript.
    let digest = proof
        .public_input
        .get_hash(proof.config.n_verifier_friendly_commitment_layers);
    // Construct the transcript.
    let mut transcript = Transcript::new(digest);

    // STARK commitment phase.
    let stark_commitment = stark_commit::<Layout>(
        &mut transcript,
        &proof.public_input,
        &proof.unsent_commitment,
        &proof.config,
        &stark_domains,
    )?;

    let state_const = commitment_to_const_state(&stark_commitment.fri);

    // Generate queries.
    let queries = generate_queries(
        &mut transcript,
        proof.config.n_queries,
        stark_domains.eval_domain_size,
    );

    // STARK verify phase.
    let step_iter = generate_step_iter::<Layout>(
        n_original_columns,
        n_interaction_columns,
        &proof.public_input,
        &queries,
        stark_commitment,
        &proof.witness,
        &stark_domains,
    );

    let mut proof: StarkProofWithSerde = proof.into();
    proof.witness.fri_witness.layers.clear();

    Ok(SplitProof {
        proof,
        state_const,
        step_iter,
    })
}

fn generate_step_iter<Layout: LayoutTrait>(
    n_original_columns: u32,
    n_interaction_columns: u32,
    public_input: &PublicInput,
    queries: &[Felt],
    commitment: StarkCommitment<Layout::InteractionElements>,
    witness: &StarkWitness,
    stark_domains: &StarkDomains,
) -> VerifyProofStepParamIter {
    // Compute query points.
    let points = queries_to_points(queries, stark_domains);

    // Evaluate the FRI input layer at query points.
    let eval_info = OodsEvaluationInfo {
        oods_values: commitment.oods_values,
        oods_point: commitment.interaction_after_composition,
        trace_generator: stark_domains.trace_generator,
        constraint_coefficients: commitment.interaction_after_oods,
    };
    let oods_poly_evals = eval_oods_boundary_poly_at_points::<Layout>(
        n_original_columns,
        n_interaction_columns,
        public_input,
        &eval_info,
        &points,
        &witness.traces_decommitment,
        &witness.composition_decommitment,
    );

    VerifyProofStepParamIter {
        next_index: 0,
        n_layers: (commitment.fri.config.n_layers - 1).try_into().unwrap(),
        next_queries: gather_first_layer_queries(queries, oods_poly_evals, points),
        fri_group: get_fri_group(),
        layer_witness: witness.fri_witness.layers.to_owned(),
        eval_points: commitment.fri.eval_points,
        step_sizes: commitment.fri.config.fri_step_sizes[1..].to_vec(),
        last_layer_coefficients: commitment.fri.last_layer_coefficients,
    }
}
fn commitment_to_const_state(
    commitment: &swiftness_fri::types::Commitment,
) -> FriVerificationStateConstant {
    FriVerificationStateConstant {
        n_layers: (commitment.config.n_layers - Felt::ONE).try_into().unwrap(),
        commitment: commitment
            .inner_layers
            .iter()
            .map(|layer| TableCommitment {
                config: TableCommitmentConfig {
                    n_columns: layer.config.n_columns,
                    vector: VectorCommitmentConfig {
                        height: layer.config.vector.height,
                        n_verifier_friendly_commitment_layers: layer
                            .config
                            .vector
                            .n_verifier_friendly_commitment_layers,
                    },
                },
                vector_commitment: VectorCommitment {
                    config: VectorCommitmentConfig {
                        height: layer.vector_commitment.config.height,
                        n_verifier_friendly_commitment_layers: layer
                            .vector_commitment
                            .config
                            .n_verifier_friendly_commitment_layers,
                    },
                    commitment_hash: layer.vector_commitment.commitment_hash,
                },
            })
            .collect(),
        eval_points: commitment.eval_points.clone(),
        step_sizes: commitment
            .config
            .fri_step_sizes
            .iter()
            .skip(1)
            .cloned()
            .collect(),
        last_layer_coefficients_hash: poseidon_hash_many(&commitment.last_layer_coefficients),
    }
}
