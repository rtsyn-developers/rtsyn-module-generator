# RTSyn Module Template Generator

Generate a new RTSyn module from a small module specification.

## Usage

```bash
xmake
xmake run rtsyn-module-generator examples/adder.toml ../rtsyn-adder-generated
```

Pass `--force` to replace an existing output directory.

```bash
xmake run rtsyn-module-generator -- --force examples/adder.toml ../rtsyn-adder-generated
```

## Spec

The spec uses a small TOML subset with root keys and repeated sections:

```toml
name = "adder"
kind = "plugin"
description = "Adds two numeric inputs and writes the result."
author = "Sergio Hidalgo"

[[ports]]
name = "left"
number = 0
direction = "input"
type = "f64"
description = "Left operand."

[[params]]
name = "gain"
number = 0
type = "f64"
default = "1.0"
description = "Gain applied by the module."

[[states]]
name = "result"
number = 0
type = "f64"
description = "Most recently computed result."
```

`[[values]]` is accepted as an alias for `[[params]]`.

Root `kind` selects the generated module type:

- `plugin`: generates from `templates/rtsyn-module-plugin-template`.
- `device`: generates from `templates/rtsyn-module-device-template`.

If omitted, `kind` defaults to `plugin`.

## Templates

The generator reads templates from:

- `templates/rtsyn-module-plugin-template`
- `templates/rtsyn-module-device-template`

Those directories are intended to be the corresponding template repositories
mounted as git submodules.
