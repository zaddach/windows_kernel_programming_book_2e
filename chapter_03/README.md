# Chapter 3

This is a user-space program to interface with the "Beep" device. It uses the `windows` crate to
interface with the Win32 and Nt API. In particular, you can see the usage of `NtOpenFile` to access
the device without a symlink.

The project was set up with the command:
```ps1
cargo new --bin chapter_03
cd chapter_03
cargo add windows
```

The `windows` crate features have been figured out with the documentation for each API call.

Note how in the code, e.g., `NTSTATUS` is defined differently from the type in `wdk-sys`: In
userspace, it is a newtype pattern which implements additional methods such as `.is_ok()` on the
type.
