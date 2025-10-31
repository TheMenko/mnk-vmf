# VMF Parser

A high-performance, zero-copy parser for Valve Map Format (VMF) files written in Rust.

## Overview

VMF (Valve Map Format) is a text-based file format used by Source Engine level editors like Hammer to store map data. This library provides a fast, memory-efficient parser that leverages memory-mapped I/O and zero-copy parsing techniques to handle large map files with minimal overhead.

## Features

- **Zero-copy parsing** - String data references the memory-mapped file directly, avoiding allocations
- **Memory-efficient** - Uses `memmap2` for lazy-loaded file access
- **Type-safe** - Strongly-typed representations of all VMF structures
- **Flexible parsing** - Parse individual blocks or entire files

## Supported VMF Structures

- Version information and metadata
- World geometry (solids, brushes, sides)
- Entities (point entities, brush entities)
- Displacement surfaces (terrain)
- Visibility groups
- View settings
- Cameras
- Cordons

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vmf = "0.1.0"
```

## Usage

### Basic Example

```rust
use vmf::VMF;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open and parse a VMF file
    let vmf = VMF::open("map.vmf")?;
    let data = vmf.parse()?;

    // Iterate through parsed blocks
    for value in data {
        match value {
            VMFValue::World(world) => {
                println!("World contains {} solids", world.solids.len());
            }
            VMFValue::Entity(entity) => {
                println!("Entity: {:?}", entity.classname);
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Parsing Individual Components

You can also parse specific VMF blocks directly:

```rust
use vmf::Parser;
use vmf::types::VersionInfo;

let input = r#"
versioninfo
{
    "editorversion" "400"
    "editorbuild" "6157"
    "mapversion" "16"
    "formatversion" "100"
    "prefab" "0"
}
"#;

let version_info = VersionInfo::parse(input)?;
println!("Editor version: {}", version_info.editor_version);
```

### Working with World Geometry

```rust
use vmf::{VMF, VMFValue};

let vmf = VMF::open("map.vmf")?;
let data = vmf.parse()?;

for value in data {
    if let VMFValue::World(world) = value {
        for solid in &world.solids {
            println!("Solid ID: {}", solid.id);
            for side in &solid.sides {
                println!("  Material: {}", side.material);
                println!("  Plane: {:?}", side.plane);
            }
        }
    }
}
```

## Performance

The parser is designed for performance:

- Memory-mapped I/O reduces memory footprint
- Zero-copy string parsing eliminates allocation overhead
- Benchmarks included for all major components
- Suitable for large map files (tested with files over 100MB)

Run benchmarks with:

```bash
cargo bench
```

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
