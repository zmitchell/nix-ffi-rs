# nix-ffi-rs

This project is a set of experimental Rust bindings to Nix.

The project contains two parts:
1. A set of bindings to the C++ implementation using [cxx](https://github.com/dtolnay/cxx).
2. A set of bindings to the proposed Nix C API from [nix#8699](https://github.com/NixOS/nix/pull/8699).

Both projects have their own flakes, although there's probably a way to unify them and make the two projects part of a Rust workspace.

### `nix-cxx`
This project takes a hardcoded string, parses it as a flake reference, then prints out the parsed flake reference.
In short, it round trips a string through Nix.

### `nix-bindgen`
This project evaluates Nix code provided in one of a few ways:
- A string passed to the `-e` flag.
- A filename passed as the only positional argument.
- A string piped to `stdin`.

The evaluation result is printed, but the program will crash if the expression doesn't evaluate to a string.
This isn't a problem with the API, I just stopped working on this once I got evaluation working.

### Expectations

- Quality
    - Weekend project
- Accepting contributions?
    - Not really, this is a proof of concept. If you want to improve the infrastructure around this proof of concept (the flakes, the Rust workspace, etc), that would be more than welcome.
- Stable?
    - `nix-cxx` should be relatively stable since it doesn't rely on much of the Nix internals.
    - `nix-bindgen` is "lol no" level of stability. The flake takes a PR commit as an input, so a rebase means this won't build anymore since that commit will no longer exist.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
