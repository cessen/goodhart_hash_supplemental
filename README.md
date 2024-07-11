# Hash Design and Goodhart's Law - supplemental code

This is the code used to compute the data used in the final table of [Hash Design and Goodhart's Law](https://blog.cessen.com/post/2024_07_10_hash_design_and_goodharts_law).  Please see that article for context.

Note that this only runs on x86-64 due to use of x86-64 intrinsics in some of the code.

## Building and Running

To build, first ensure that you have [Rust](https://www.rust-lang.org) installed.  Then from the root of this repo:

```
cargo build --release
```

To run:

```
cargo run --release
```

Some of the tests can take a little while to run.
