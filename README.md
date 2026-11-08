<div align="center">

# PROJECT VELOCITY

**A Rust/Bevy rewrite of Project Velocity**

</div>

Project Velocity was a street-racing game by Visual Sciences for Sierra on the original Xbox, cancelled in 2005.
This is a function-by-function rewrite of its debug build in Rust on Bevy. It runs on PC straight from the
original game files: no assets are included or converted, you need your own copy.

> **Unfinished.** It boots, races, and runs the menus, career and movies, but there are known bugs, some missing
> pieces, and performance work in progress.

## Building

Requires Rust (the toolchain is pinned in `rust-toolchain.toml`, MSVC on Windows).

```
cargo build --release -p velocity
```

Put `target/release/velocity.exe` next to `default.xbe` in the game folder, or run it with
`velocity.exe --game <path to game folder>`.
