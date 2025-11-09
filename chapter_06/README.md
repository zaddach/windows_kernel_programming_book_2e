# Kernel Mechanisms

## Structured Exception Handling
Rust doesn't have native support for Structured Exception Handling (SEH). There's [microseh](https://crates.io/crates/microseh), a crate that add support for SEH to Rust, but it does have drawbacks. Since the
exception handling isn't integrated with Rust's stack unwinding, `::drop()` will not be called in case
an exception causes the executed closure to abort (As documented [here](https://github.com/sonodima/microseh/blob/5e28ed907b9a9454d5b6d60ff3367f60ac80340d/examples/raii.rs#L20-L32)). So be very careful to only
wrap calls to native functions that might raise SEH exceptions, and to avoid any resource allocation
in Rust code inside the closure.

If you can avoid structured exceptions at all, that's probably a better idea.