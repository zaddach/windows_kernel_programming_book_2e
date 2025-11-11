# Overview

This chapter takes the code from [chapter 4](../chapter_04/README.md) and brings
some improvements:
- Use trace logging instead of debug prints.

# Projects

## boost
Nothing new here.

## booster
We've added the [tracelogging](https://crates.io/crates/tracelogging) crate with
features "kernel_mode" and "macros".

Then we can declare the tracelogging provider with `define_provider!(...)`,
and log trace events with `write_event!(...)`.

## booster2
The original implementation uses C variadic functions for implementing `Log`,
`LogInfo` and `LogError`. While it is technically possible to implement C variadic functions
in Rust nightly with feature [c_variadic](https://doc.rust-lang.org/beta/unstable-book/language-features/c-variadic.html), I opted to implement the logging with declarative macros. This approach doesn't
require a nightly toolchain and the same convenience.

