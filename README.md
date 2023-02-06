# `hacker`

`hacker` is an assembler that converts Hack assembly code into binary machine code as defined in the [nand2tetris] project.  Please follow the instructions below to install `hacker`.

## Installing Git

Please run the command
```
git -v
```
to ensure you have `git` installed.

## Installing Rust

`hacker` is implemented in the [Rust] programming language.  We recommended installing Rust via the official installer [`rustup`].  If you already have Rust installed, please run the command
```
cargo -V
```
to check if your installed version is at least `1.66.1`.  If this is not the case, please run the command
```
rustup update
```
to update your Rust installation.

## Installing `hacker`

1. Clone the source of `hacker` with Git:
    ```
    git clone https://github.com/Fanatic-Provender/hacker.git
    cd hacker
    ```

2. Build `hacker` using Cargo:
    ```
    cargo build --release
    ```

## Using `hacker`

In the `hacker` directory, run the command
```
cargo run -- <FILE>
```
to convert the assembly file `<FILE>` to machine code.
`<FILE>` should have the extension `asm`,
and `hacker` will generate an output file with the extension `hack`.
The output file will rewrite any existing file with the same name.


[nand2tetris]: https://www.nand2tetris.org/
[Rust]: https://www.rust-lang.org/
[`rustup`]: https://rustup.rs/
