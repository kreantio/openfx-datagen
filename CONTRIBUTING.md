# CONTRIBUTING

## AI Policy

See [AI_POLICY.md](./AI_POLICY.md).

## Terminology

This project refers to properties in these ways:

- **cname**[^1]: the C `#define` name.
- **stringname**[^2]: the actual string corresponding to the cname.
- **canonical name**[^3]: `cname.strip_prefix("k")`
- **simple name**[^4]: `cname.strip_prefix("kOfx")`

Read [CONTRIBUTING.md§Terminology in openfx-rs] for more details.

[^1]: https://github.com/AcademySoftwareFoundation/openfx/blob/e40728885390ec16276d11e00025de9b4282060c/scripts/gen-props.py#L215

[^2]: https://github.com/AcademySoftwareFoundation/openfx/blob/e40728885390ec16276d11e00025de9b4282060c/scripts/gen-props.py#L236

[^3]: made up by Umaĵo.

[^4]: https://github.com/AcademySoftwareFoundation/openfx/blob/e40728885390ec16276d11e00025de9b4282060c/scripts/gen-props.py#L192

[CONTRIBUTING.md§Terminology in openfx-rs]: https://github.com/kreantio/openfx-rs/blob/main/CONTRIBUTING.md#terminology
