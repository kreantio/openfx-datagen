# `openfx-datagen`

This is a subproject of [`openfx-rs`].

See Also:

- [AI_POLICY.md](./AI_POLICY.md)
- [ATTRIBUTION.md](./ATTRIBUTION.md)

`openfx-datagen` is a tool for generating data from the official OpenFX C
headers[^1].

This repository also contains the code for the `openfx-bindgen` crate, which
generates Rust bindings from data produced by `openfx-datagen`.

[official repo]: https://github.com/AcademySoftwareFoundation/openfx

[^1]: They are usually located under the `/include` folder in the
    [official repo].

## Prerequisites

The following tools are required:

| Tool(s) \\ To                           | Run tests | Update Generated Schemata |
| --------------------------------------- | --------- | ------------------------- |
| POSIX tools (`sh`, `rm`, `mkdir`, etc.) | no        | not strictly              |
| [`just`]                                | no        | not strictly              |

[`openfx-rs`]: https://github.com/kreantio/openfx-rs
[`just`]: https://github.com/casey/just
