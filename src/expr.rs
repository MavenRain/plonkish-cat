//! Symbolic polynomial expressions over wires.
//!
//! [`Expression<F>`] is the free algebra over wire references and
//! field constants.  Expressions are built symbolically during
//! constraint generation and evaluated against wire assignments
//! for satisfaction checking.

use crate::error::Error;
use crate::field::Field;
use crate::wire::Wire;

/// A symbolic polynomial expression over field `F` and wire indices.
///
/// Used to build constraints: an expression that must equal zero.
#[derive(Debug, Clone)]
pub enum Expression<F: Field> {
    /// A field constant.
    Constant(F),
    /// A wire reference (a variable).
    Wire(Wire),
    /// Negation of an expression.
    Neg(Box<Expression<F>>),
    /// Sum of two expressions.
    Sum(Box<Expression<F>>, Box<Expression<F>>),
    /// Product of two expressions.
    Product(Box<Expression<F>>, Box<Expression<F>>),
}

impl<F: Field> Expression<F> {
    /// A constant expression.
    #[must_use]
    pub fn constant(c: F) -> Self {
        Self::Constant(c)
    }

    /// A wire reference.
    #[must_use]
    pub fn wire(w: Wire) -> Self {
        Self::Wire(w)
    }

    /// Evaluate this expression given a wire-value assignment.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WireOutOfBounds`] if a referenced wire
    /// is not in the assignment.
    pub fn evaluate(&self, assignment: &dyn Fn(Wire) -> Result<F, Error>) -> Result<F, Error> {
        match self {
            Self::Constant(c) => Ok(c.clone()),
            Self::Wire(w) => assignment(*w),
            Self::Neg(inner) => inner.evaluate(assignment).map(|v| -v),
            Self::Sum(left, right) => {
                let l = left.evaluate(assignment)?;
                let r = right.evaluate(assignment)?;
                Ok(l + r)
            }
            Self::Product(left, right) => {
                let l = left.evaluate(assignment)?;
                let r = right.evaluate(assignment)?;
                Ok(l * r)
            }
        }
    }
}

impl<F: Field> std::ops::Add for Expression<F> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field> std::ops::Sub for Expression<F> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl<F: Field> std::ops::Mul for Expression<F> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::Product(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field> std::ops::Neg for Expression<F> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::Neg(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::F101;

    fn test_assignment(w: Wire) -> Result<F101, Error> {
        match w.index() {
            0 => Ok(F101::new(3)),
            1 => Ok(F101::new(5)),
            2 => Ok(F101::new(7)),
            _ => Err(Error::WireOutOfBounds {
                wire_index: w.index(),
                allocated: 3,
            }),
        }
    }

    #[test]
    fn constant_evaluates_to_itself() -> Result<(), Error> {
        let e = Expression::constant(F101::new(42));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(42));
        Ok(())
    }

    #[test]
    fn wire_evaluates_to_assignment() -> Result<(), Error> {
        let e = Expression::wire(Wire::new(1));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(5));
        Ok(())
    }

    #[test]
    fn sum_evaluates_correctly() -> Result<(), Error> {
        let e = Expression::wire(Wire::new(0)) + Expression::wire(Wire::new(1));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(8));
        Ok(())
    }

    #[test]
    fn product_evaluates_correctly() -> Result<(), Error> {
        let e = Expression::wire(Wire::new(0)) * Expression::wire(Wire::new(1));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(15));
        Ok(())
    }

    #[test]
    fn subtraction_evaluates_correctly() -> Result<(), Error> {
        // 3 - 5 = -2 = 99 (mod 101)
        let e = Expression::wire(Wire::new(0)) - Expression::wire(Wire::new(1));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(99));
        Ok(())
    }

    #[test]
    fn negation_evaluates_correctly() -> Result<(), Error> {
        // -3 = 98 (mod 101)
        let e = -Expression::wire(Wire::new(0));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(98));
        Ok(())
    }

    #[test]
    fn complex_expression() -> Result<(), Error> {
        // w0 * w1 + w2 = 3 * 5 + 7 = 22
        let e = Expression::wire(Wire::new(0)) * Expression::wire(Wire::new(1))
            + Expression::wire(Wire::new(2));
        assert_eq!(e.evaluate(&test_assignment)?, F101::new(22));
        Ok(())
    }
}
