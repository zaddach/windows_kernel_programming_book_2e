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
