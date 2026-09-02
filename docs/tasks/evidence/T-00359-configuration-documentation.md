# T-00359 — Dependency & Toolchain Pinning / configuration: Documentation

## Documentation Strategy

The configuration files introduced during this epic serve as self-documenting enforcement constraints for the repository:
- `config/toolchain.json`: Contains explicit key-value pairs (`rust_version`, `python_version`, `node_version`, `enforce_hashes`) that specify the AIOS toolchain pinning footprint.
- `rust-toolchain.toml`, `.python-version`, `.nvmrc`: Adhere strictly to upstream ecosystem standards (Cargo, Pyenv/uv, NVM). Developers and CI pipelines native to these ecosystems inherently understand how to parse and obey these constraints.

No separate markdown file is needed to explain `.python-version`, as the standard format is globally recognized. The CLI tool `aiosh toolchain check` validates these values automatically during CI pipelines and agent execution.
