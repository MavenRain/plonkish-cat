//! Minimal field trait and test field.
//!
//! A finite field provides the algebraic operations needed for
//! circuit constraints.  The trait uses `std::ops` supertraits
//! for arithmetic, adding only `zero`, `one`, and `inv`.

use crate::error::Error;

/// A finite field element.
///
/// Implementations must satisfy the field axioms: associativity,
/// commutativity, distributivity of multiplication over addition,
/// and existence of additive and multiplicative inverses.
pub trait Field:
    Sized
    + Clone
    + PartialEq
    + Eq
    + core::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Neg<Output = Self>
{
    /// The additive identity.
    #[must_use]
    fn zero() -> Self;

    /// The multiplicative identity.
    #[must_use]
    fn one() -> Self;

    /// Multiplicative inverse.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DivisionByZero`] if `self` is zero.
    fn inv(&self) -> Result<Self, Error>;
}

/// The field of integers modulo 101.
///
/// A prime field with characteristic 101, suitable for testing
/// circuit constraints with small, inspectable values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct F101(u64);

impl F101 {
    /// Create a new field element, reducing modulo 101.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value % 101)
    }

    /// The underlying integer value in `[0, 101)`.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for F101 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for F101 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self((self.0 + rhs.0) % 101)
    }
}

impl std::ops::Sub for F101 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self((self.0 + 101 - rhs.0) % 101)
    }
}

impl std::ops::Mul for F101 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self((self.0 * rhs.0) % 101)
    }
}

impl std::ops::Neg for F101 {
    type Output = Self;
    fn neg(self) -> Self {
        Self((101 - self.0) % 101)
    }
}

impl Field for F101 {
    fn zero() -> Self {
        Self(0)
    }

    fn one() -> Self {
        Self(1)
    }

    fn inv(&self) -> Result<Self, Error> {
        if self.0 == 0 {
            Err(Error::DivisionByZero)
        } else {
            // Fermat's little theorem: a^{-1} = a^{p-2} mod p
            Ok(pow_mod(self.0, 99, 101))
        }
    }
}

/// Modular exponentiation by squaring: `base^exp mod modulus`.
fn pow_mod(base: u64, exp: u64, modulus: u64) -> F101 {
    // Iterative squaring via fold over the bits of `exp`.
    let result = (0..64)
        .filter(|i| (exp >> i) & 1 == 1)
        .fold((1u64, base), |(acc, _), i| {
            // Compute base^(2^i) mod modulus
            let power = (0..i).fold(base, |b, _| (b * b) % modulus);
            ((acc * power) % modulus, base)
        })
        .0;
    F101(result % modulus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_additive_identity() {
        let a = F101::new(42);
        assert_eq!(a + F101::zero(), a);
        assert_eq!(F101::zero() + a, a);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let a = F101::new(42);
        assert_eq!(a * F101::one(), a);
        assert_eq!(F101::one() * a, a);
    }

    #[test]
    fn additive_inverse() {
        let a = F101::new(42);
        assert_eq!(a + (-a), F101::zero());
    }

    #[test]
    fn multiplicative_inverse() -> Result<(), Error> {
        let a = F101::new(42);
        let a_inv = a.inv()?;
        assert_eq!(a * a_inv, F101::one());
        Ok(())
    }

    #[test]
    fn inverse_of_zero_fails() {
        let result = F101::zero().inv();
        assert!(result.is_err());
    }

    #[test]
    fn all_nonzero_elements_have_inverses() -> Result<(), Error> {
        for i in 1..101 {
            let a = F101::new(i);
            let a_inv = a.inv()?;
            assert_eq!(a * a_inv, F101::one(), "failed for {i}");
        }
        Ok(())
    }

    #[test]
    fn subtraction_is_add_neg() {
        let a = F101::new(50);
        let b = F101::new(30);
        assert_eq!(a - b, a + (-b));
    }

    #[test]
    fn multiplication_is_commutative() {
        let a = F101::new(7);
        let b = F101::new(13);
        assert_eq!(a * b, b * a);
    }

    #[test]
    fn distributivity() {
        let a = F101::new(3);
        let b = F101::new(5);
        let c = F101::new(7);
        assert_eq!(a * (b + c), a * b + a * c);
    }

    #[test]
    fn new_reduces_mod_101() {
        assert_eq!(F101::new(202), F101::new(0));
        assert_eq!(F101::new(103), F101::new(2));
    }
}
