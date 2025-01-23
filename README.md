<p align="center">
  <h1 align="center">integrity-rs</h1>
</p>

**Rust library for verifying STARK proofs from [`swiftness`](https://github.com/iosis-tech/swiftness) on [`integrity`](https://github.com/HerodotusDev/integrity)**

## Introduction

`integrity-rs` is the missing piece for verifying [`swiftness`](https://github.com/iosis-tech/swiftness) STARK proofs on-chain using the [`integrity`](https://github.com/HerodotusDev/integrity) verifier contract.

Given a STARK proof, the library offers a `split_proof` function that generates contract calls which stay under Starknet transaction size limits, allowing the proof to be verified in a multi-step process over multiple transactions.

An [example](./examples/split_proof.rs) of reading a JSON proof file and generating the final contract calls is available for reference.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
