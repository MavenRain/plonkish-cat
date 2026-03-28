//! Circuit compilation: paths to constraint sets.
//!
//! The [`compile`] function walks a path in the `PLONKish` free category,
//! allocating fresh wires for each gate boundary and generating
//! constraints.  This is the universal property of the free category
//! in action: a graph morphism (gate interpretations) extending to
//! a functor on composed circuits.

use comp_cat_rs::collapse::free_category::Path;

use crate::constraint::{Constraint, CopyConstraint, ConstraintSet};
use crate::error::Error;
use crate::expr::Expression;
use crate::field::Field;
use crate::gate::{PlonkishGraph, PrimitiveGate};
use crate::wire::{WireAllocator, WireCount, WireRange};

/// Interpret a single gate given input and output wire ranges.
///
/// Generates the constraints for this gate.
fn interpret_gate<F: Field>(
    gate: &PrimitiveGate<F>,
    input: &WireRange,
    output: &WireRange,
) -> Result<ConstraintSet<F>, Error> {
    match gate {
        PrimitiveGate::Add => {
            // out = in0 + in1  =>  out - in0 - in1 = 0
            let in0 = Expression::wire(input.wire_at(0)?);
            let in1 = Expression::wire(input.wire_at(1)?);
            let out = Expression::wire(output.wire_at(0)?);
            Ok(ConstraintSet::empty()
                .with_constraint(Constraint::new(out - (in0 + in1))))
        }
        PrimitiveGate::Mul => {
            // out = in0 * in1  =>  out - in0 * in1 = 0
            let in0 = Expression::wire(input.wire_at(0)?);
            let in1 = Expression::wire(input.wire_at(1)?);
            let out = Expression::wire(output.wire_at(0)?);
            Ok(ConstraintSet::empty()
                .with_constraint(Constraint::new(out - in0 * in1)))
        }
        PrimitiveGate::Const(c) => {
            // out = c  =>  out - c = 0
            let out = Expression::wire(output.wire_at(0)?);
            Ok(ConstraintSet::empty()
                .with_constraint(Constraint::new(out - Expression::constant(c.clone()))))
        }
        PrimitiveGate::Bool => {
            // in * (1 - in) = 0, and out = in (copy constraint)
            let inp = Expression::wire(input.wire_at(0)?);
            let one = Expression::constant(F::one());
            let bool_check = inp.clone() * (one - inp);
            Ok(ConstraintSet::empty()
                .with_constraint(Constraint::new(bool_check))
                .with_copy(CopyConstraint::new(
                    input.wire_at(0)?,
                    output.wire_at(0)?,
                )))
        }
        PrimitiveGate::Dup => {
            // Both outputs equal the input (copy constraints).
            let inp = input.wire_at(0)?;
            let out0 = output.wire_at(0)?;
            let out1 = output.wire_at(1)?;
            Ok(ConstraintSet::empty()
                .with_copy(CopyConstraint::new(inp, out0))
                .with_copy(CopyConstraint::new(inp, out1)))
        }
    }
}

/// The state threaded through the compilation fold.
struct CompileState<F: Field> {
    allocator: WireAllocator,
    constraints: ConstraintSet<F>,
    /// The output wire range of the previous gate (input to the next).
    prev_output: Option<WireRange>,
}

/// Compile a path in the free category to a constraint set.
///
/// Walks the path, allocating fresh wires for each gate boundary,
/// generating constraints for each gate, and merging them.
///
/// For an identity path, returns an empty constraint set.
///
/// # Errors
///
/// Returns an error if the path references invalid edges or
/// wire counts are mismatched.
pub fn compile<F: Field>(
    graph: &PlonkishGraph<F>,
    path: &Path,
) -> Result<ConstraintSet<F>, Error> {
    if path.is_empty() {
        return Ok(ConstraintSet::empty());
    }

    let initial = CompileState {
        allocator: WireAllocator::new(),
        constraints: ConstraintSet::empty(),
        prev_output: None,
    };

    let result = path
        .edges()
        .iter()
        .try_fold(initial, |state, edge| -> Result<CompileState<F>, Error> {
            let spec = graph.gate_spec_at(*edge)?;
            let gate = spec.gate();

            // Allocate input wires: reuse previous output, or allocate fresh.
            let (input, allocator) = state.prev_output.map_or_else(
                || -> Result<(WireRange, WireAllocator), Error> {
                    let input_count = graph.wire_count_at(spec.source())?;
                    Ok(state.allocator.allocate(input_count))
                },
                |prev| Ok((prev, state.allocator)),
            )?;

            // Allocate output wires.
            let output_count = graph.wire_count_at(spec.target())?;
            let (output, allocator) = allocator.allocate(output_count);

            // Generate constraints for this gate.
            let gate_constraints = interpret_gate(gate, &input, &output)?;

            Ok(CompileState {
                allocator,
                constraints: state.constraints.merge(gate_constraints),
                prev_output: Some(output),
            })
        })?;

    Ok(result.constraints)
}

/// Compile a path and return both the constraint set and the
/// input/output wire ranges.
///
/// Useful for testing: the input range is the first gate's source
/// wires, the output range is the last gate's target wires.
///
/// # Errors
///
/// Returns an error if compilation fails.
pub fn compile_with_io<F: Field>(
    graph: &PlonkishGraph<F>,
    path: &Path,
) -> Result<(ConstraintSet<F>, WireRange, WireRange), Error> {
    if path.is_empty() {
        let empty_range = WireRange::new(crate::wire::Wire::new(0), WireCount::new(0));
        return Ok((ConstraintSet::empty(), empty_range, empty_range));
    }

    let initial_input_count = path
        .edges()
        .first()
        .map_or(Ok(WireCount::new(0)), |e| {
            let spec = graph.gate_spec_at(*e)?;
            graph.wire_count_at(spec.source())
        })?;

    let allocator = WireAllocator::new();
    let (input_range, allocator) = allocator.allocate(initial_input_count);

    let initial = CompileState {
        allocator,
        constraints: ConstraintSet::empty(),
        prev_output: Some(input_range),
    };

    let result = path
        .edges()
        .iter()
        .try_fold(initial, |state, edge| -> Result<CompileState<F>, Error> {
            let spec = graph.gate_spec_at(*edge)?;
            let gate = spec.gate();

            let input = state.prev_output.unwrap_or(
                WireRange::new(crate::wire::Wire::new(0), WireCount::new(0)),
            );

            let output_count = graph.wire_count_at(spec.target())?;
            let (output, allocator) = state.allocator.allocate(output_count);

            let gate_constraints = interpret_gate(gate, &input, &output)?;

            Ok(CompileState {
                allocator,
                constraints: state.constraints.merge(gate_constraints),
                prev_output: Some(output),
            })
        })?;

    let output_range = result.prev_output.unwrap_or(
        WireRange::new(crate::wire::Wire::new(0), WireCount::new(0)),
    );

    Ok((result.constraints, input_range, output_range))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::F101;
    use crate::wire::Wire;
    use comp_cat_rs::collapse::free_category::{Edge, Path};

    fn make_assignment(values: &[(usize, u64)]) -> impl Fn(Wire) -> Result<F101, Error> + '_ {
        move |w| {
            values
                .iter()
                .find(|(i, _)| *i == w.index())
                .map(|(_, v)| F101::new(*v))
                .ok_or(Error::WireOutOfBounds {
                    wire_index: w.index(),
                    allocated: values.len(),
                })
        }
    }

    #[test]
    fn compile_identity_path() -> Result<(), Error> {
        let graph = PlonkishGraph::<F101>::standard();
        let path = Path::identity(comp_cat_rs::collapse::free_category::Vertex::new(1));
        let cs = compile(&graph, &path)?;
        assert_eq!(cs.constraints().len(), 0);
        assert_eq!(cs.copy_constraints().len(), 0);
        Ok(())
    }

    #[test]
    fn compile_single_add_gate() -> Result<(), Error> {
        let graph = PlonkishGraph::<F101>::standard();
        // Edge 0 = Add: vertex 2 -> vertex 1
        let path = Path::singleton(&graph, Edge::new(0))?;
        let (cs, input, output) = compile_with_io(&graph, &path)?;

        assert_eq!(input.count(), WireCount::new(2));
        assert_eq!(output.count(), WireCount::new(1));
        assert_eq!(cs.constraints().len(), 1);

        // 3 + 5 = 8: wires w0=3, w1=5, w2=8
        let assignment = make_assignment(&[(0, 3), (1, 5), (2, 8)]);
        assert!(cs.is_satisfied(&assignment)?);

        // Wrong output: should fail
        let bad_assignment = make_assignment(&[(0, 3), (1, 5), (2, 7)]);
        assert!(!cs.is_satisfied(&bad_assignment)?);

        Ok(())
    }

    #[test]
    fn compile_single_mul_gate() -> Result<(), Error> {
        let graph = PlonkishGraph::<F101>::standard();
        // Edge 1 = Mul: vertex 2 -> vertex 1
        let path = Path::singleton(&graph, Edge::new(1))?;
        let cs = compile(&graph, &path)?;

        // 3 * 5 = 15: wires w0=3, w1=5, w2=15
        let assignment = make_assignment(&[(0, 3), (1, 5), (2, 15)]);
        assert!(cs.is_satisfied(&assignment)?);
        Ok(())
    }

    #[test]
    fn compile_dup_then_mul_is_square() -> Result<(), Error> {
        // dup ; mul : 1 -> 2 -> 1
        // Input x, output x^2
        let graph = PlonkishGraph::<F101>::standard();
        let dup = Path::singleton(&graph, Edge::new(3))?; // Dup: 1 -> 2
        let mul = Path::singleton(&graph, Edge::new(1))?; // Mul: 2 -> 1
        let square = dup.compose(mul)?;

        let (cs, input, output) = compile_with_io(&graph, &square)?;

        assert_eq!(input.count(), WireCount::new(1));
        assert_eq!(output.count(), WireCount::new(1));

        // Dup generates 2 copy constraints (w0 -> w1, w0 -> w2)
        assert_eq!(cs.copy_constraints().len(), 2);
        // Mul generates 1 polynomial constraint (w3 - w1*w2 = 0)
        assert_eq!(cs.constraints().len(), 1);

        // x=3, x^2=9: w0=3, w1=3, w2=3, w3=9
        let assignment = make_assignment(&[(0, 3), (1, 3), (2, 3), (3, 9)]);
        assert!(cs.is_satisfied(&assignment)?);

        // Wrong: x=3, output=10
        let bad = make_assignment(&[(0, 3), (1, 3), (2, 3), (3, 10)]);
        assert!(!bad_satisfied(&cs, &bad)?);

        Ok(())
    }

    fn bad_satisfied(
        cs: &ConstraintSet<F101>,
        assignment: &dyn Fn(Wire) -> Result<F101, Error>,
    ) -> Result<bool, Error> {
        cs.is_satisfied(assignment)
    }

    #[test]
    fn compile_const_gate() -> Result<(), Error> {
        let graph = PlonkishGraph::<F101>::standard();
        let (graph, const_edge) = graph.with_const(F101::new(42));

        let path = Path::singleton(&graph, const_edge)?;
        let cs = compile(&graph, &path)?;

        // Output wire must equal 42
        let assignment = make_assignment(&[(0, 42)]);
        assert!(cs.is_satisfied(&assignment)?);

        let bad = make_assignment(&[(0, 41)]);
        assert!(!cs.is_satisfied(&bad)?);

        Ok(())
    }

    #[test]
    fn compile_bool_gate() -> Result<(), Error> {
        let graph = PlonkishGraph::<F101>::standard();
        // Edge 2 = Bool: vertex 1 -> vertex 1
        let path = Path::singleton(&graph, Edge::new(2))?;
        let cs = compile(&graph, &path)?;

        // 0 is boolean
        let zero = make_assignment(&[(0, 0), (1, 0)]);
        assert!(cs.is_satisfied(&zero)?);

        // 1 is boolean
        let one = make_assignment(&[(0, 1), (1, 1)]);
        assert!(cs.is_satisfied(&one)?);

        // 2 is not boolean
        let two = make_assignment(&[(0, 2), (1, 2)]);
        assert!(!cs.is_satisfied(&two)?);

        Ok(())
    }
}
