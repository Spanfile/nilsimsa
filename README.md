# nilsimsa

Implementation of the Nilsimsa locality-sensitive hashing algorithm.

Compared to "traditional" hash functions (cryptographic or not), a small modification to the input does not substantially change the resulting hash. This crate contains the Nilsimsa utility to calculate Nilsimsa hash digests, as well as a `compare` function for given digests.

## Usage

```rust
let mut hasher = Nilsimsa::new();
hasher.update("input string");
hasher.update("more strings");
let digest = hasher.digest();
```
