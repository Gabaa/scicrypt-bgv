_This is a part of **scicrypt**. For more information, head to the
[scicrypt](https://crates.io/crates/scicrypt) crate homepage._

This crate implements a `BigInteger`, for which most arithmetic operations take a constant amount of time given the specified sizes. This crate is nothing more than a convenient wrapper around the low-level constant-time functions from GMP.

If the crate is not working as expected, consider running with `--debug` to hit the `debug_assert!()`s, to check that the preconditions are met for some of the low-level functions.

Exceptions:
- Initializing from strings using `from_string`
- Equality checks (`PartialEq`)
- Ordering (`PartialOrd`)
- `lcm_leaky`
- `mod_u_leaky`
- `set_bit_leaky` and `clear_bit_leaky`
- `is_probably_prime_leaky`


We are unsure about random number generation.
Also note that division may leak some information about the size of the resulting value.