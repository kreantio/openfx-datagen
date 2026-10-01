# `openfx-datagen`

See Also:

- [AI_POLICY.md](./AI_POLICY.md)
- [ATTRIBUTION.md](./ATTRIBUTION.md)

`openfx-datagen` is a tool for generating data from the official OpenFX C
headers[^1].

[official repo]: https://github.com/AcademySoftwareFoundation/openfx

[^1]: They are usually located under the `/include` folder in the
    [official repo].

## Prerequisites

The following tools are required:

| Tool(s)                                 | Run tests | Updating Generated Schemata |
| --------------------------------------- | --------- | --------------------------- |
| POSIX tools (`sh`, `rm`, `mkdir`, etc.) | no        | not strictly required       |
| [`just`]                                | no        | not strictly required       |

[`just`]: https://github.com/casey/just
