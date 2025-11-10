//! # mnk-vmf
//!
//! A fast parser for Valve Map Format (VMF) files used in Source engine.
//!
//! This crate provides efficient parsing of VMF files into a strongly-typed AST, with support
//! for all major VMF constructs including worlds, entities, solids, displacements, and more.
//!
//! ## Quick Start
//!
//! ```ignore
//! use mnk_vmf::VMF;
//!
//! let vmf = VMF::open("mymap.vmf")?;
//! let data = vmf.parse()?;
//!
//! for block in data {
//!     match block {
//!         mnk_vmf::VMFValue::World(world) => {
//!             println!("World has {} solids", world.solids.len());
//!         }
//!         mnk_vmf::VMFValue::Entity(entity) => {
//!             println!("Entity: {}", entity.classname);
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - **Fast parsing**: Uses Chumsky parser combinators for efficient token-based parsing
//! - **Complete VMF support**: Handles versioninfo, visgroups, worlds, entities, solids, displacements, cameras, and more
//! - **Strong typing**: All VMF constructs are represented as Rust types with proper error handling
//!
//! ## Modules
//!
//! - [`vmf`]: Main entry point for loading and parsing VMF files
//! - [`types`]: All VMF data types (World, Entity, Solid, etc.)
//! - [`parser`]: Low-level parsing utilities and traits

mod error;
mod parser;
pub mod types;
pub mod vmf;

pub use parser::Parser;
pub use parser::util;
pub use vmf::*;
