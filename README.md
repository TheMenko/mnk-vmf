# VMF Parser

A high-performance parser for Valve Map Format (VMF) files written in Rust.

## Overview

VMF (Valve Map Format) is a text-based file format used by Source Engine level editors like Hammer to store map data. This library provides a fast, memory-efficient parser.

## Supported VMF Structures

- Version information and metadata
- World geometry (solids, brushes, sides)
- Entities (point entities, brush entities)
- Displacement surfaces (terrain)
- Visibility groups
- View settings
- Cameras
- Cordons

## Usage

```toml
[dependencies]
mnk_vmf = "0.1.0"
```

### Basic Example

```rust
use mnk_vmf::VMF;

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

### Working with World Geometry

```rust
use mnk_vmf::{VMF, VMFValue};

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

Licensed under either of

  * Apache License, Version 2.0, (http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license (http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
