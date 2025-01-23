//! `integrity-rs` is the missing piece for verifying
//! [`swiftness`](https://github.com/iosis-tech/swiftness) STARK proofs on-chain using the
//! [`integrity`](https://github.com/HerodotusDev/integrity) verifier contract.
//!
//! Given a STARK proof, the library offers a `split_proof` function that generates contract calls
//! which stay under Starknet transaction size limits, allowing the proof to be verified in a
//! multi-step process over multiple transactions.

pub use starknet_core::{
    codec::{Decode, Encode},
    types::Felt,
};

use starknet_core::types::Call;

/// Bindings for the `integrity` contract.
pub mod bindings;
pub use bindings::{
    StarkProofWithSerde, VerifierConfiguration, VerifyProofFinalAndRegisterFactCall,
    VerifyProofInitialCall, VerifyProofStepCall,
};

mod split;
pub use split::{split_proof, SplitProof, VerifyProofStepParamIter};

/// Contract bindings for all contract calls needed to verify a STARK proof on-chain.
#[derive(Debug, Clone)]
pub struct IntegrityCalls {
    /// The initial verification call.
    pub initial: VerifyProofInitialCall,
    /// The intermediate verification step calls.
    pub intermediate_steps: Vec<VerifyProofStepCall>,
    /// The final verification call.
    pub final_step: VerifyProofFinalAndRegisterFactCall,
}

impl IntegrityCalls {
    /// Flattens the calls into a list of [`Call`] ready for use with `starknet-rs`.
    pub fn collect_calls(self, contract_address: Felt) -> Vec<Call> {
        let mut calls = vec![self.initial.call(contract_address)];
        calls.extend(
            self.intermediate_steps
                .into_iter()
                .map(|step| step.call(contract_address)),
        );
        calls.push(self.final_step.call(contract_address));
        calls
    }
}
