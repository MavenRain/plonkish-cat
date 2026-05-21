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
//! use field_cat::F101;
//! use comp_cat_rs::collapse::free_category::{Edge, Path};
//!
//! // Build the standard `PLONKish` graph.
//! let graph = PlonkishGraph::<F101>::standard();
//!
//! // Compose: dup (1 -> 2) then mul (2 -> 1) = squaring circuit.
//! let dup = Path::singleton(&graph, Edge::new(3))?;
//! let mul = Path::singleton(&graph, Edge::new(1))?;
//! let square = dup.compose(mul)?;
//!
//! // Compile to constraints.
//! let _constraints = compile(&graph, &square)?;
//! # Ok::<(), plonkish_cat::Error>(())
//! ```
//!
//! The [`Field`](field_cat::Field) trait and the toy
//! [`F101`](field_cat::F101) field used in this example live in
//! the sibling [`field_cat`] crate so they can be shared with
//! STARK-flavored downstreams without inheriting the `PLONKish`
//! constraint vocabulary.

pub mod constraint;
pub mod error;
pub mod expr;
pub mod gate;
pub mod interpret;
pub mod wire;

pub use constraint::{Constraint, ConstraintSet, CopyConstraint};
pub use error::Error;
pub use expr::Expression;
pub use gate::{PlonkishGraph, PrimitiveGate};
pub use interpret::compile;
pub use wire::{Wire, WireAllocator, WireCount, WireRange};
