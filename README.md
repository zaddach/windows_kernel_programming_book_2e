# Windows Kernel Programming - 2nd Edition

![book cover](https://d2sofvawe08yqg.cloudfront.net/windowskernelprogrammingsecondedition/s_hero2x?1620653873)

I'm reading Pavel Yosifovich's excellent book [Windows Kernel Programming - 2nd Edition](https://leanpub.com/windowskernelprogrammingsecondedition), which is teaching the theoretical OS knowledge and practical implementation of drivers in C++.

Since Microsoft seems to be headed towards a future of Windows drivers written in Rust (see [here](https://techcommunity.microsoft.com/blog/surfaceitpro/safer-drivers-stronger-devices/4431411)), I wanted to convert the [book's exercise code](https://github.com/zodiacon/windowskernelprogrammingbook2e) to Rust code as a little finger-flexing exercise.

## Getting Started

The build environment setup process is explained in the [windows-drivers-rs Getting Started](https://github.com/microsoft/windows-drivers-rs#getting-started) section:

- Install LLVM: `winget install -i LLVM.LLVM`
- Install `cargo cmake`: `cargo install --locked cargo-make --no-default-features --features tls-native`

## Chapters
- [Chapter 2](./chapter_02/README.md)


