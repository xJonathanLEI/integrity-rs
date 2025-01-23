use integrity::{split_proof, Felt, VerifierConfiguration};
use starknet_macros::short_string;
use swiftness::TransformTo;
use swiftness_stark::types::StarkProof;

/// Sepolia `integrity` verifier contract address.
const VERIFIER: Felt =
    Felt::from_hex_unchecked("0x04ce7851f00b6c3289674841fd7a1b96b6fd41ed1edc248faccd672c26371b8c");

fn main() {
    // Parse proof from JSON file
    let proof: StarkProof = swiftness::parse(std::fs::read_to_string("./proof.json").unwrap())
        .unwrap()
        .transform_to();

    // Split proof into multiple steps
    let proof = split_proof::<swiftness_air::layout::recursive::Layout>(proof).unwrap();

    // Configure the calls by supplying a unique job ID and verifier config
    let calls = proof.into_calls(
        short_string!("random_job_id"),
        VerifierConfiguration {
            layout: short_string!("recursive"),
            hasher: short_string!("keccak_160_lsb"),
            stone_version: short_string!("stone6"),
            memory_verification: short_string!("cairo1"),
        },
    );

    // Flatten the calls into a regular `Vec<Call>` ready for use with `starknet-rs`
    let calls = calls.collect_calls(VERIFIER);
    println!("{} contract calls generated: {:#?}", calls.len(), calls);
}
