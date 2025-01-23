pub use starknet_core::{
    codec::{Decode, Encode},
    types::Felt,
};

use starknet_core::{
    codec::{Error as CodecError, FeltWriter},
    types::Call,
};
use swiftness::{
    config::StarkConfig as SwiftnessStarkConfig,
    types::{
        StarkUnsentCommitment as SwiftnessStarkUnsentCommitment,
        StarkWitness as SwiftnessStarkWitness,
    },
};
use swiftness_air::{
    public_memory::PublicInput as SwiftnessPublicInput,
    trace::{
        config::Config as SwiftnessTracesConfig, Decommitment as SwiftnessTracesDecommitment,
        UnsentCommitment as SwiftnessTracesUnsentCommitment, Witness as SwiftnessTracesWitness,
    },
    types::{
        AddrValue as SwiftnessAddrValue, ContinuousPageHeader as SwiftnessContinuousPageHeader,
        SegmentInfo as SwiftnessSegmentInfo,
    },
};
use swiftness_commitment::{
    table::{
        config::Config as SwiftnessTableCommitmentConfig,
        types::{
            Decommitment as SwiftnessTableDecommitment, Witness as SwiftnessTableCommitmentWitness,
        },
    },
    vector::{
        config::Config as SwiftnessVectorCommitmentConfig,
        types::Witness as SwiftnessVectorCommitmentWitness,
    },
};
use swiftness_fri::{
    config::Config as SwiftnessFriConfig,
    types::{UnsentCommitment as SwiftnessFriUnsentCommitment, Witness as SwiftnessFriWitness},
};
use swiftness_pow::{
    config::Config as SwiftnessProofOfWorkConfig,
    pow::UnsentCommitment as SwiftnessProofOfWorkUnsentCommitment,
};
use swiftness_stark::types::StarkProof as SwiftnessStarkProof;

/// Entrypoint selector for `verify_proof_initial`.
const SELECTOR_VERIFY_PROOF_INITIAL_CALL: Felt = Felt::from_raw([
    454550947884470974,
    16477582295426715492,
    11685118883294889452,
    4530997181248663582,
]);

/// Entrypoint selector for `verify_proof_step`.
const SELECTOR_VERIFY_PROOF_STEP_CALL: Felt = Felt::from_raw([
    366928098735624260,
    14431289083207541201,
    10380245210905814816,
    18299522247102387854,
]);

/// Entrypoint selector for `verify_proof_final_and_register_fact`.
const SELECTOR_VERIFY_PROOF_FINAL_AND_REGISTER_FACT_CALL: Felt = Felt::from_raw([
    123220592339497,
    16622672023924009708,
    11528706201916495377,
    6934812503915115676,
]);

/// Contract binding for the `verify_proof_initial` contract entrypoint.
#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VerifyProofInitialCall {
    pub job_id: Felt,
    pub verifier_config: VerifierConfiguration,
    pub stark_proof: StarkProofWithSerde,
}

/// Contract binding for the `verify_proof_step` contract entrypoint.
#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VerifyProofStepCall {
    pub job_id: Felt,
    pub state_constant: FriVerificationStateConstant,
    pub state_variable: FriVerificationStateVariable,
    pub witness: FriLayerWitness,
}

/// Contract binding for the `verify_proof_final_and_register_fact` contract entrypoint.
#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VerifyProofFinalAndRegisterFactCall {
    pub job_id: Felt,
    pub state_constant: FriVerificationStateConstant,
    pub state_variable: FriVerificationStateVariable,
    pub last_layer_coefficients: Vec<Felt>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VerifierConfiguration {
    pub layout: Felt,
    pub hasher: Felt,
    pub stone_version: Felt,
    pub memory_verification: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct StarkProofWithSerde {
    pub config: StarkConfigWithSerde,
    pub public_input: PublicInputWithSerde,
    pub unsent_commitment: StarkUnsentCommitmentWithSerde,
    pub witness: StarkWitnessWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct StarkConfigWithSerde {
    pub traces: TracesConfigWithSerde,
    pub composition: TableCommitmentConfigWithSerde,
    pub fri: FriConfigWithSerde,
    pub proof_of_work: ProofOfWorkConfigWithSerde,
    pub log_trace_domain_size: Felt,
    pub n_queries: Felt,
    pub log_n_cosets: Felt,
    pub n_verifier_friendly_commitment_layers: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TracesConfigWithSerde {
    pub original: TableCommitmentConfigWithSerde,
    pub interaction: TableCommitmentConfigWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TableCommitmentConfigWithSerde {
    pub n_columns: Felt,
    pub vector: VectorCommitmentConfigWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VectorCommitmentConfigWithSerde {
    pub height: Felt,
    pub n_verifier_friendly_commitment_layers: Felt,
}

#[derive(Debug, Clone)]
pub struct FriConfigWithSerde {
    pub log_input_size: Felt,
    pub n_layers: Felt,
    pub inner_layers: Vec<TableCommitmentConfigWithSerde>,
    pub fri_step_sizes: Vec<Felt>,
    pub log_last_layer_degree_bound: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct ProofOfWorkConfigWithSerde {
    pub n_bits: u8,
}

#[derive(Debug, Clone)]
pub struct PublicInputWithSerde {
    pub log_n_steps: Felt,
    pub range_check_min: Felt,
    pub range_check_max: Felt,
    pub layout: Felt,
    pub dynamic_params: Vec<Felt>,
    pub segments: Vec<SegmentInfo>,
    pub padding_addr: Felt,
    pub padding_value: Felt,
    pub main_page: Vec<AddrValue>,
    pub continuous_page_headers: Vec<ContinuousPageHeader>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct SegmentInfo {
    pub begin_addr: Felt,
    pub stop_ptr: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct AddrValue {
    pub address: Felt,
    pub value: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct ContinuousPageHeader {
    pub start_address: Felt,
    pub size: Felt,
    pub hash: Felt,
    pub prod: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct StarkUnsentCommitmentWithSerde {
    pub traces: TracesUnsentCommitmentWithSerde,
    pub composition: Felt,
    pub oods_values: Vec<Felt>,
    pub fri: FriUnsentCommitmentWithSerde,
    pub proof_of_work: ProofOfWorkUnsentCommitmentWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TracesUnsentCommitmentWithSerde {
    pub original: Felt,
    pub interaction: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriUnsentCommitmentWithSerde {
    pub inner_layers: Vec<Felt>,
    pub last_layer_coefficients: Vec<Felt>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct ProofOfWorkUnsentCommitmentWithSerde {
    pub nonce: u64,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct StarkWitnessWithSerde {
    pub traces_decommitment: TracesDecommitmentWithSerde,
    pub traces_witness: TracesWitnessWithSerde,
    pub composition_decommitment: TableDecommitmentWithSerde,
    pub composition_witness: TableCommitmentWitnessWithSerde,
    pub fri_witness: FriWitnessWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TracesDecommitmentWithSerde {
    pub original: TableDecommitmentWithSerde,
    pub interaction: TableDecommitmentWithSerde,
}

#[derive(Debug, Clone)]
pub struct TableDecommitmentWithSerde {
    pub values: Vec<Felt>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TracesWitnessWithSerde {
    pub original: TableCommitmentWitnessWithSerde,
    pub interaction: TableCommitmentWitnessWithSerde,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TableCommitmentWitnessWithSerde {
    pub vector: VectorCommitmentWitnessWithSerde,
}

#[derive(Debug, Clone)]
pub struct VectorCommitmentWitnessWithSerde {
    pub authentications: Vec<Felt>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriWitnessWithSerde {
    pub layers: Vec<Felt>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriVerificationStateConstant {
    pub n_layers: u32,
    pub commitment: Vec<TableCommitment>,
    pub eval_points: Vec<Felt>,
    pub step_sizes: Vec<Felt>,
    pub last_layer_coefficients_hash: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriVerificationStateVariable {
    pub iter: u32,
    pub queries: Vec<FriLayerQuery>,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriLayerWitness {
    pub leaves: Vec<Felt>,
    pub table_witness: TableCommitmentWitness,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TableCommitment {
    pub config: TableCommitmentConfig,
    pub vector_commitment: VectorCommitment,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TableCommitmentConfig {
    pub n_columns: Felt,
    pub vector: VectorCommitmentConfig,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VectorCommitment {
    pub config: VectorCommitmentConfig,
    pub commitment_hash: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VectorCommitmentConfig {
    pub height: Felt,
    pub n_verifier_friendly_commitment_layers: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct FriLayerQuery {
    pub index: Felt,
    pub y_value: Felt,
    pub x_inv_value: Felt,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct TableCommitmentWitness {
    pub vector: VectorCommitmentWitness,
}

#[derive(Debug, Clone, Encode)]
#[starknet(core = "starknet_core")]
pub struct VectorCommitmentWitness {
    pub authentications: Vec<Felt>,
}

impl VerifyProofInitialCall {
    pub fn call(&self, contract_address: Felt) -> Call {
        Call {
            to: contract_address,
            selector: SELECTOR_VERIFY_PROOF_INITIAL_CALL,
            calldata: self.calldata(),
        }
    }

    pub fn calldata(&self) -> Vec<Felt> {
        let mut calldata = vec![];

        // This type never fails to serialize
        self.encode(&mut calldata).unwrap();

        calldata
    }
}

impl VerifyProofStepCall {
    pub fn call(&self, contract_address: Felt) -> Call {
        Call {
            to: contract_address,
            selector: SELECTOR_VERIFY_PROOF_STEP_CALL,
            calldata: self.calldata(),
        }
    }

    pub fn calldata(&self) -> Vec<Felt> {
        let mut calldata = vec![];

        // This type never fails to serialize
        self.encode(&mut calldata).unwrap();

        calldata
    }
}

impl VerifyProofFinalAndRegisterFactCall {
    pub fn call(&self, contract_address: Felt) -> Call {
        Call {
            to: contract_address,
            selector: SELECTOR_VERIFY_PROOF_FINAL_AND_REGISTER_FACT_CALL,
            calldata: self.calldata(),
        }
    }

    pub fn calldata(&self) -> Vec<Felt> {
        let mut calldata = vec![];

        // This type never fails to serialize
        self.encode(&mut calldata).unwrap();

        calldata
    }
}

impl From<SwiftnessStarkProof> for StarkProofWithSerde {
    fn from(value: SwiftnessStarkProof) -> Self {
        Self {
            config: value.config.into(),
            public_input: value.public_input.into(),
            unsent_commitment: value.unsent_commitment.into(),
            witness: value.witness.into(),
        }
    }
}

impl From<SwiftnessStarkConfig> for StarkConfigWithSerde {
    fn from(value: SwiftnessStarkConfig) -> Self {
        Self {
            traces: value.traces.into(),
            composition: value.composition.into(),
            fri: value.fri.into(),
            proof_of_work: value.proof_of_work.into(),
            log_trace_domain_size: value.log_trace_domain_size,
            n_queries: value.n_queries,
            log_n_cosets: value.log_n_cosets,
            n_verifier_friendly_commitment_layers: value.n_verifier_friendly_commitment_layers,
        }
    }
}

impl From<SwiftnessTracesConfig> for TracesConfigWithSerde {
    fn from(value: SwiftnessTracesConfig) -> Self {
        Self {
            original: value.original.into(),
            interaction: value.interaction.into(),
        }
    }
}

impl From<SwiftnessTableCommitmentConfig> for TableCommitmentConfigWithSerde {
    fn from(value: SwiftnessTableCommitmentConfig) -> Self {
        Self {
            n_columns: value.n_columns,
            vector: value.vector.into(),
        }
    }
}

impl From<SwiftnessVectorCommitmentConfig> for VectorCommitmentConfigWithSerde {
    fn from(value: SwiftnessVectorCommitmentConfig) -> Self {
        Self {
            height: value.height,
            n_verifier_friendly_commitment_layers: value.n_verifier_friendly_commitment_layers,
        }
    }
}

impl From<SwiftnessFriConfig> for FriConfigWithSerde {
    fn from(value: SwiftnessFriConfig) -> Self {
        Self {
            log_input_size: value.log_input_size,
            n_layers: value.n_layers,
            inner_layers: value.inner_layers.into_iter().map(|l| l.into()).collect(),
            fri_step_sizes: value.fri_step_sizes,
            log_last_layer_degree_bound: value.log_last_layer_degree_bound,
        }
    }
}

impl From<SwiftnessProofOfWorkConfig> for ProofOfWorkConfigWithSerde {
    fn from(value: SwiftnessProofOfWorkConfig) -> Self {
        Self {
            n_bits: value.n_bits,
        }
    }
}

impl From<SwiftnessPublicInput> for PublicInputWithSerde {
    fn from(value: SwiftnessPublicInput) -> Self {
        let dynamic_params = match value.dynamic_params {
            Some(dynamic_params) => Vec::<u32>::from(dynamic_params)
                .into_iter()
                .map(Felt::from)
                .collect(),
            None => vec![],
        };

        Self {
            log_n_steps: value.log_n_steps,
            range_check_min: value.range_check_min,
            range_check_max: value.range_check_max,
            layout: value.layout,
            dynamic_params,
            segments: value.segments.into_iter().map(Into::into).collect(),
            padding_addr: value.padding_addr,
            padding_value: value.padding_value,
            main_page: value.main_page.0.into_iter().map(Into::into).collect(),
            continuous_page_headers: value
                .continuous_page_headers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<SwiftnessSegmentInfo> for SegmentInfo {
    fn from(value: SwiftnessSegmentInfo) -> Self {
        Self {
            begin_addr: value.begin_addr,
            stop_ptr: value.stop_ptr,
        }
    }
}

impl From<SwiftnessAddrValue> for AddrValue {
    fn from(value: SwiftnessAddrValue) -> Self {
        Self {
            address: value.address,
            value: value.value,
        }
    }
}

impl From<SwiftnessContinuousPageHeader> for ContinuousPageHeader {
    fn from(value: SwiftnessContinuousPageHeader) -> Self {
        Self {
            start_address: value.start_address,
            size: value.size,
            hash: value.hash,
            prod: value.prod,
        }
    }
}

impl From<SwiftnessStarkUnsentCommitment> for StarkUnsentCommitmentWithSerde {
    fn from(value: SwiftnessStarkUnsentCommitment) -> Self {
        Self {
            traces: value.traces.into(),
            composition: value.composition,
            oods_values: value.oods_values,
            fri: value.fri.into(),
            proof_of_work: value.proof_of_work.into(),
        }
    }
}

impl From<SwiftnessTracesUnsentCommitment> for TracesUnsentCommitmentWithSerde {
    fn from(value: SwiftnessTracesUnsentCommitment) -> Self {
        Self {
            original: value.original,
            interaction: value.interaction,
        }
    }
}

impl From<SwiftnessFriUnsentCommitment> for FriUnsentCommitmentWithSerde {
    fn from(value: SwiftnessFriUnsentCommitment) -> Self {
        Self {
            inner_layers: value.inner_layers,
            last_layer_coefficients: value.last_layer_coefficients,
        }
    }
}

impl From<SwiftnessProofOfWorkUnsentCommitment> for ProofOfWorkUnsentCommitmentWithSerde {
    fn from(value: SwiftnessProofOfWorkUnsentCommitment) -> Self {
        Self { nonce: value.nonce }
    }
}

impl From<SwiftnessStarkWitness> for StarkWitnessWithSerde {
    fn from(value: SwiftnessStarkWitness) -> Self {
        Self {
            traces_decommitment: value.traces_decommitment.into(),
            traces_witness: value.traces_witness.into(),
            composition_decommitment: value.composition_decommitment.into(),
            composition_witness: value.composition_witness.into(),
            fri_witness: value.fri_witness.into(),
        }
    }
}

impl From<SwiftnessTracesDecommitment> for TracesDecommitmentWithSerde {
    fn from(value: SwiftnessTracesDecommitment) -> Self {
        Self {
            original: value.original.into(),
            interaction: value.interaction.into(),
        }
    }
}

impl From<SwiftnessTableDecommitment> for TableDecommitmentWithSerde {
    fn from(value: SwiftnessTableDecommitment) -> Self {
        Self {
            values: value.values,
        }
    }
}

impl From<SwiftnessTracesWitness> for TracesWitnessWithSerde {
    fn from(value: SwiftnessTracesWitness) -> Self {
        Self {
            original: value.original.into(),
            interaction: value.interaction.into(),
        }
    }
}

impl From<SwiftnessTableCommitmentWitness> for TableCommitmentWitnessWithSerde {
    fn from(value: SwiftnessTableCommitmentWitness) -> Self {
        Self {
            vector: value.vector.into(),
        }
    }
}

impl From<SwiftnessVectorCommitmentWitness> for VectorCommitmentWitnessWithSerde {
    fn from(value: SwiftnessVectorCommitmentWitness) -> Self {
        Self {
            authentications: value.authentications,
        }
    }
}

impl From<SwiftnessFriWitness> for FriWitnessWithSerde {
    fn from(_value: SwiftnessFriWitness) -> Self {
        // This is technically wrong. It's done here as `layers` should be empty when splitting
        // proofs. However, the proper way would be to correctly convert the field and let the
        // caller empty the layers.
        // TODO: idiomatically convert the type
        Self { layers: vec![] }
    }
}

// Manually implementing as the canonical type uses `Vec<Felt>` for `inner_layers`, which makes no
// sense. This library uses `Vec<TableCommitmentConfigWithSerde>` instead but this custom `Encode`
// is needed to maintain the length prefix behaviour of `Vec<Felt>`.
impl Encode for FriConfigWithSerde {
    fn encode<W: FeltWriter>(&self, writer: &mut W) -> Result<(), CodecError> {
        self.log_input_size.encode(writer)?;
        self.n_layers.encode(writer)?;

        writer.write((self.inner_layers.len() * 3).into());
        for inner_layer in &self.inner_layers {
            inner_layer.encode(writer)?;
        }

        self.fri_step_sizes.encode(writer)?;
        self.log_last_layer_degree_bound.encode(writer)
    }
}

impl Encode for PublicInputWithSerde {
    fn encode<W: FeltWriter>(&self, writer: &mut W) -> Result<(), CodecError> {
        self.log_n_steps.encode(writer)?;
        self.range_check_min.encode(writer)?;
        self.range_check_max.encode(writer)?;
        self.layout.encode(writer)?;
        self.dynamic_params.encode(writer)?;

        // `n_segments`
        writer.write(self.segments.len().into());

        // `segments`
        writer.write((self.segments.len() * 2).into());
        for segment in &self.segments {
            segment.encode(writer)?;
        }

        self.padding_addr.encode(writer)?;
        self.padding_value.encode(writer)?;

        // `main_page_len`
        writer.write(self.main_page.len().into());

        // `main_page`
        writer.write((self.main_page.len() * 2).into());
        for addr_value in &self.main_page {
            addr_value.encode(writer)?;
        }

        // `n_continuous_pages`
        writer.write(self.continuous_page_headers.len().into());

        // `continuous_page_headers`
        writer.write((self.continuous_page_headers.len() * 4).into());
        for header in &self.continuous_page_headers {
            header.encode(writer)?;
        }

        Ok(())
    }
}

impl Encode for TableDecommitmentWithSerde {
    fn encode<W: FeltWriter>(&self, writer: &mut W) -> Result<(), CodecError> {
        // `n_values`
        writer.write(self.values.len().into());

        // `values`
        self.values.encode(writer)
    }
}

impl Encode for VectorCommitmentWitnessWithSerde {
    fn encode<W: FeltWriter>(&self, writer: &mut W) -> Result<(), CodecError> {
        // `n_authentications`
        writer.write(self.authentications.len().into());

        // `authentications`
        self.authentications.encode(writer)
    }
}
