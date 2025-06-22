# NEAR MPC RNG

This repo shows how to create unbiasable onchain randomness using NEAR's VRF + NEAR Chain Signatures.

It makes a cross contract call to NEAR's MPC chain signatures `sign` using NEAR's VRF `random_seed` value as a payload value.

The callback in the contract then uses the `random_seed` value combined with the returned signature's `R` and `s` values as entropy to a sha256 hash.

The resulting hash is your randomness.

## Commit Reveal Version

Originally, there was a commit/reveal version, but it doesn't offer much more security over the MPC only version.

You can find it here for reference:

https://github.com/mattlockyer/near-mpc-rng/tree/2c0b0253fa98f904128b0c650a0686d450641695
