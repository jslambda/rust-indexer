# rust2json

`rust2json` builds a JSON search index from the Rust source code in a project. It parses items in the `src/` directory and emits structured metadata that other tools can use to power search, documentation, or code navigation features.

## Installation

The tool is distributed as a Cargo binary. Install it from a local checkout with:

```bash
cargo install --path .
```

You can also run it directly with `cargo run --` without installing.

## Usage

```
rust2json [project_root]
```

- `project_root` (optional): Root of the Rust project to index. Defaults to the current working directory.
- `-h`, `--help`: Print the built-in usage text.

The CLI writes the index to standard output. Redirect it to a file if you want to save the results:

```bash
rust2json . > index.json
```

## Library usage

`rust2json` exposes a small API so you can embed indexing in your own tools. Add it as a dependency:

```toml
[dependencies]
rust2json = "0.1.0"
```

Then call the library helpers to build and serialize the index:

```rust
use std::fs::File;
use std::path::Path;

use rust2json::{build_index, write_index_to};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = Path::new(".");
    let entries = build_index(project_root)?;
    let output = File::create("index.json")?;

    write_index_to(&entries, output)?;

    Ok(())
}
```

If you want to index a single Rust source file, use `build_file_index`:

```rust
use std::path::Path;

use rust2json::build_file_index;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = build_file_index(Path::new("src/lib.rs"))?;
    println!("Indexed {} entries", entries.len());
    Ok(())
}
```

## Output schema

Each entry in the JSON array uses the following fields:

| field        | description                                                                 |
|--------------|-----------------------------------------------------------------------------|
| `kind`       | Item type such as `module`, `struct`, `enum`, `trait`, `impl`, or `fn`.      |
| `name`       | Identifier of the item.                                                     |
| `file`       | Path to the file containing the item, relative to the project root.         |
| `line_start` | Starting line number of the item.                                           |
| `line_end`   | Ending line number of the item.                                             |
| `signature`  | Rendered signature for the item (e.g., `fn` signature or `impl` header).    |
| `doc_summary`| First non-empty line of Rust doc comments, if present.                      |
| `doc`        | Full Rust doc comments joined by newlines, if present.                      |

## How it works

The indexer walks every `.rs` file under `src/`, parses it with `syn`, and extracts top-level modules, structs, enums, traits, functions, and impl blocks. Doc comments are gathered from Rust `///` attributes, and line ranges come from source spans so the index can point back to original code locations.

## Development

Run the test suite to verify changes:

```bash
cargo test
```
