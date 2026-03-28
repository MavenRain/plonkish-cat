//! Primitive gates and the `PLONKish` graph.
//!
//! The `PLONKish` graph is the generating data for the free category
//! of circuits.  Vertices are wire counts, edges are primitive gates.
//! Implements [`Graph`] from comp-cat-rs.

use comp_cat_rs::collapse::free_category::{Edge, FreeCategoryError, Graph, Vertex};

use crate::field::Field;
use crate::wire::WireCount;

/// A primitive gate in the `PLONKish` circuit.
///
/// Each gate is an edge in the circuit graph, connecting
/// a source wire count to a target wire count.
#[derive(Debug, Clone)]
pub enum PrimitiveGate<F: Field> {
    /// Addition gate: 2 input wires, 1 output wire.
    /// Constrains: `output = input_0 + input_1`
    Add,
    /// Multiplication gate: 2 input wires, 1 output wire.
    /// Constrains: `output = input_0 * input_1`
    Mul,
    /// Constant gate: 0 input wires, 1 output wire.
    /// Constrains: `output = c`
    Const(F),
    /// Boolean check gate: 1 input wire, 1 output wire.
    /// Constrains: `input * (1 - input) = 0`, `output = input`
    Bool,
    /// Duplication gate: 1 input wire, 2 output wires.
    /// Copy constraint: both outputs equal the input.
    Dup,
}

impl<F: Field> PrimitiveGate<F> {
    /// The number of input wires this gate expects.
    #[must_use]
    pub fn input_count(&self) -> WireCount {
        match self {
            Self::Add | Self::Mul => WireCount::new(2),
            Self::Const(_) => WireCount::new(0),
            Self::Bool | Self::Dup => WireCount::new(1),
        }
    }

    /// The number of output wires this gate produces.
    #[must_use]
    pub fn output_count(&self) -> WireCount {
        match self {
            Self::Add | Self::Mul | Self::Const(_) | Self::Bool => WireCount::new(1),
            Self::Dup => WireCount::new(2),
        }
    }
}

/// A gate specification: a gate together with its source and target vertices.
#[derive(Debug, Clone)]
pub struct GateSpec<F: Field> {
    gate: PrimitiveGate<F>,
    source: Vertex,
    target: Vertex,
}

impl<F: Field> GateSpec<F> {
    /// Create a new gate specification.
    #[must_use]
    pub fn new(gate: PrimitiveGate<F>, source: Vertex, target: Vertex) -> Self {
        Self {
            gate,
            source,
            target,
        }
    }

    /// The gate.
    #[must_use]
    pub fn gate(&self) -> &PrimitiveGate<F> {
        &self.gate
    }

    /// The source vertex (input wire count).
    #[must_use]
    pub fn source(&self) -> Vertex {
        self.source
    }

    /// The target vertex (output wire count).
    #[must_use]
    pub fn target(&self) -> Vertex {
        self.target
    }
}

/// The `PLONKish` circuit graph: vertices are wire counts,
/// edges are primitive gates.
///
/// Implements [`Graph`] from comp-cat-rs, enabling construction
/// of [`Path`](comp_cat_rs::collapse::free_category::Path)s
/// (composed circuits) in the free category.
#[derive(Debug)]
pub struct PlonkishGraph<F: Field> {
    vertices: Vec<WireCount>,
    edges: Vec<GateSpec<F>>,
}

impl<F: Field> PlonkishGraph<F> {
    /// Build the standard `PLONKish` graph with the basic gate set.
    ///
    /// Vertices:
    ///   0 -> `WireCount(0)` (empty, unit object)
    ///   1 -> `WireCount(1)` (single wire)
    ///   2 -> `WireCount(2)` (pair of wires)
    ///
    /// Edges:
    ///   0: Add   (vertex 2 -> vertex 1)
    ///   1: Mul   (vertex 2 -> vertex 1)
    ///   2: Bool  (vertex 1 -> vertex 1)
    ///   3: Dup   (vertex 1 -> vertex 2)
    #[must_use]
    pub fn standard() -> Self {
        Self {
            vertices: vec![WireCount::new(0), WireCount::new(1), WireCount::new(2)],
            edges: vec![
                GateSpec::new(PrimitiveGate::Add, Vertex::new(2), Vertex::new(1)),
                GateSpec::new(PrimitiveGate::Mul, Vertex::new(2), Vertex::new(1)),
                GateSpec::new(PrimitiveGate::Bool, Vertex::new(1), Vertex::new(1)),
                GateSpec::new(PrimitiveGate::Dup, Vertex::new(1), Vertex::new(2)),
            ],
        }
    }

    /// Add a constant gate for a specific field element.
    ///
    /// Returns the extended graph and the edge index of the new gate.
    #[must_use]
    pub fn with_const(self, c: F) -> (Self, Edge) {
        let edge_index = Edge::new(self.edges.len());
        let new_spec = GateSpec::new(PrimitiveGate::Const(c), Vertex::new(0), Vertex::new(1));
        let graph = Self {
            vertices: self.vertices,
            edges: self
                .edges
                .into_iter()
                .chain(core::iter::once(new_spec))
                .collect(),
        };
        (graph, edge_index)
    }

    /// The vertices (wire counts) in this graph.
    #[must_use]
    pub fn vertices(&self) -> &[WireCount] {
        &self.vertices
    }

    /// The gate specifications (edges) in this graph.
    #[must_use]
    pub fn gate_specs(&self) -> &[GateSpec<F>] {
        &self.edges
    }

    /// Look up the gate specification for an edge.
    ///
    /// # Errors
    ///
    /// Returns an error if the edge index is out of bounds.
    pub fn gate_spec_at(&self, edge: Edge) -> Result<&GateSpec<F>, crate::error::Error> {
        if edge.index() < self.edges.len() {
            Ok(&self.edges[edge.index()])
        } else {
            Err(FreeCategoryError::EdgeOutOfBounds {
                edge,
                count: self.edges.len(),
            }
            .into())
        }
    }

    /// Look up the wire count for a vertex.
    ///
    /// # Errors
    ///
    /// Returns an error if the vertex index is out of bounds.
    pub fn wire_count_at(&self, vertex: Vertex) -> Result<WireCount, crate::error::Error> {
        if vertex.index() < self.vertices.len() {
            Ok(self.vertices[vertex.index()])
        } else {
            Err(FreeCategoryError::VertexOutOfBounds {
                vertex,
                count: self.vertices.len(),
            }
            .into())
        }
    }
}

impl<F: Field> Graph for PlonkishGraph<F> {
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn source(&self, edge: Edge) -> Result<Vertex, FreeCategoryError> {
        if edge.index() < self.edges.len() {
            Ok(self.edges[edge.index()].source())
        } else {
            Err(FreeCategoryError::EdgeOutOfBounds {
                edge,
                count: self.edges.len(),
            })
        }
    }

    fn target(&self, edge: Edge) -> Result<Vertex, FreeCategoryError> {
        if edge.index() < self.edges.len() {
            Ok(self.edges[edge.index()].target())
        } else {
            Err(FreeCategoryError::EdgeOutOfBounds {
                edge,
                count: self.edges.len(),
            })
        }
    }
}
