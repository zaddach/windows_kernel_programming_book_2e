# Chapter 2

A minimal WDM example driver.

## Setting the project up
I'm using the freshly installed `cargo-wdk` tool to set up a new project. As the original project
from Pavel Yosifovich, we want to start a WDM driver.

```ps1
cargo wdk new --wdm chapter_02
```

After generation, I have updated the `wdk-sys` crate to its latest version, `0.4.0`.

## Writing code
The code tries to stay very close to the original C++ implementation.

The memory tag cannot be initialized as a wide character literal `'dcba'`, but has to be decoded
from ASCII string bytes instead.
```rust
const DRIVER_TAG: u32 = u32::from_ne_bytes(*b"dcba");
```

As the `Default` trait isn't const, we cannot use its implementation to initialize global static
values. We need to use the full initializer instead.
```rust
static mut REGISTRY_PATH: UNICODE_STRING = UNICODE_STRING {
   Length: 0,
   MaximumLength: 0,
   Buffer: core::ptr::null_mut(),
};
```

The `ExAllocatePoolWithTag` function is deprecated and isn't available in `wdk-sys`.
We're using the `ExAllocatePool2` function instead, as [recommended by Microsoft](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/updating-deprecated-exallocatepool-calls).

## Setting up the .inf
TODO

## Building
Run
```ps1
cargo make
```
in a terminal with administrative privileges to build and sign the driver.






