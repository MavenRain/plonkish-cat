//! # plonkish-cat
//!
//! A `PLONKish` circuit system built on comp-cat-rs's categorical
//! infrastructure.
//!
//! Circuits form a **symmetric monoidal category** where:
//! - Objects = wire counts
//! - Morphisms = gadgets (constraint-generating sub-circuits)
//! - Composition = sequential wiring
//! - Tensor product = parallel placement
//! - Braiding = wire permutation
//!
//! The free category on the `PLONKish` graph of primitive gates
//! provides the circuit DSL.  Paths are composed circuits.
//! The [`compile`] function interprets paths into constraint sets
//! via the universal property of free categories.
//!
//! ## Quick start
//!
//! ```rust
//! use plonkish_cat::*;
//! use comp_cat_rs::collapse::free_category::{Edge, Path};
//!
//! // Build the standard `PLONKish` graph.
//! let graph = PlonkishGraph::<F101>::standard();
//!
//! // Compose: dup (1 -> 2) then mul (2 -> 1) = squaring circuit.
//! let dup = Path::singleton(&graph, Edge::new(3)).unwrap();
//! let mul = Path::singleton(&graph, Edge::new(1)).unwrap();
//! let square = dup.compose(mul).unwrap();
//!
//! // Compile to constraints.
//! let constraints = compile(&graph, &square).unwrap();
//! ```

pub mod error;
pub mod field;
pub mod wire;
pub mod expr;
pub mod constraint;
pub mod gate;
pub mod interpret;

pub use error::Error;
pub use field::{F101, Field};
pub use wire::{Wire, WireAllocator, WireCount, WireRange};
pub use expr::Expression;
pub use constraint::{Constraint, CopyConstraint, ConstraintSet};
pub use gate::{PlonkishGraph, PrimitiveGate};
pub use interpret::compile;
