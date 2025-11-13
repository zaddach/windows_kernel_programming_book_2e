# Windows Kernel Programming - 2nd Edition

![book cover](https://d2sofvawe08yqg.cloudfront.net/windowskernelprogrammingsecondedition/s_hero2x?1620653873)

I'm reading Pavel Yosifovich's excellent book [Windows Kernel Programming - 2nd Edition](https://leanpub.com/windowskernelprogrammingsecondedition), which is teaching the theoretical OS knowledge and practical implementation of drivers in C++.

Since Microsoft seems to be headed towards a future of Windows drivers written in Rust (see [here](https://techcommunity.microsoft.com/blog/surfaceitpro/safer-drivers-stronger-devices/4431411)), I felt that translating the [book's exercise code](https://github.com/zodiacon/windowskernelprogrammingbook2e) to Rust code as a little finger-flexing exercise would be fun.

Since the Rust support for Windows drivers is evolving, so will this repository.

## Getting Started

The build environment setup process is explained in the [windows-drivers-rs Getting Started](https://github.com/microsoft/windows-drivers-rs#getting-started) section:

- Install LLVM: `winget install -i LLVM.LLVM`
- Install `cargo cmake`: `cargo install --locked cargo-make --no-default-features --features tls-native`

For easier scaffolding, you can install `cargo-wdk`. There's a bit of explanation in Nate Deisinger's post [Towards Rust in Windows Drivers](https://techcommunity.microsoft.com/blog/windowsdriverdev/towards-rust-in-windows-drivers/4449718) on `cargo-wdk`.
```
cargo install cargo-wdk
```

## Chapters
- [Chapter 2](./chapter_02/README.md)
- [Chapter 3](./chapter_03/README.md)
- [Chapter 4](./chapter_04/README.md)
- [Chapter 5](./chapter_05/README.md)
- [Chapter 6: Kernel Mechanisms](./chapter_06/README.md)
- [Chapter 7](./chapter_07/README.md)

## Other Rust driver resources
The [Windows-rust-driver-samples](https://github.com/microsoft/Windows-rust-driver-samples) repo is a
Rust port of the original [Windows-driver-samples](https://github.com/microsoft/Windows-driver-samples)
repository.

## Notes

### Custom Allocators
Allocation with custom allocators is currently a bit awkward in Rust. You have the possibility to either change the global allocator, or use a custom allocator with a special interface in library data types. In the Windows kernel, where you might want to allocate from the non-paged and paged pools with different tags, just having one allocator might not be enough.

There is a cool [blog post from Yoshua Wuyts in 2023](https://blog.yoshuawuyts.com/nesting-allocators/), that refreshes an idea from [Tyler Mandry from 2021](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) on scoped allocators.

That idea would allow binding an allocator to a certain scope (i.e., having a "stack" of allocators), where allocations within this scope would be serviced by the scope's allocator.


