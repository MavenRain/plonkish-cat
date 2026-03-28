//! Project-wide error type.

use comp_cat_rs::collapse::free_category::FreeCategoryError;

/// All errors that can arise in plonkish-cat.
#[derive(Debug)]
pub enum Error {
    /// An error from the free category infrastructure.
    FreeCategory(FreeCategoryError),
    /// Wire index out of bounds during allocation or constraint generation.
    WireOutOfBounds {
        /// The wire index that was accessed.
        wire_index: usize,
        /// The total number of allocated wires.
        allocated: usize,
    },
    /// Gate applied to a vertex with wrong wire count.
    WireCountMismatch {
        /// The expected wire count.
        expected: usize,
        /// The actual wire count.
        actual: usize,
    },
    /// Attempted to invert zero in the field.
    DivisionByZero,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FreeCategory(e) => write!(f, "free category error: {e}"),
            Self::WireOutOfBounds {
                wire_index,
                allocated,
            } => write!(
                f,
                "wire index {wire_index} out of bounds (allocated: {allocated})"
            ),
            Self::WireCountMismatch { expected, actual } => {
                write!(f, "wire count mismatch: expected {expected}, got {actual}")
            }
            Self::DivisionByZero => write!(f, "division by zero"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FreeCategory(e) => Some(e),
            Self::WireOutOfBounds { .. }
            | Self::WireCountMismatch { .. }
            | Self::DivisionByZero => None,
        }
    }
}

impl From<FreeCategoryError> for Error {
    fn from(e: FreeCategoryError) -> Self {
        Self::FreeCategory(e)
    }
}
