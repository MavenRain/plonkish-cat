//! Wire newtypes: the objects and allocated resources of the circuit category.
//!
//! - [`Wire`]: a single wire index (identifies a field-element carrier)
//! - [`WireCount`]: number of wires (the category's objects)
//! - [`WireRange`]: a contiguous block of allocated wires
//! - [`WireAllocator`]: functional state for fresh wire allocation

use crate::error::Error;

/// A wire index: identifies a single wire in a circuit.
///
/// Wires carry field elements at evaluation time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wire(usize);

impl Wire {
    /// Create a new wire identifier.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// The underlying index.
    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}

impl core::fmt::Display for Wire {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "w{}", self.0)
    }
}

/// A wire count: the number of wires in a bundle.
///
/// This is the "object" type in the circuit category.
/// Morphisms go from one wire count to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireCount(usize);

impl WireCount {
    /// Create a new wire count.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self(n)
    }

    /// The underlying count.
    #[must_use]
    pub fn count(self) -> usize {
        self.0
    }

    /// The zero wire count (unit object in the monoidal category).
    #[must_use]
    pub fn zero() -> Self {
        Self(0)
    }

    /// Tensor product: parallel composition of wire bundles.
    #[must_use]
    pub fn tensor(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Add for WireCount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.tensor(rhs)
    }
}

impl core::fmt::Display for WireCount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A contiguous range of wire indices.
///
/// Represents an allocated block of wires during interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireRange {
    start: Wire,
    count: WireCount,
}

impl WireRange {
    /// Create a new wire range.
    #[must_use]
    pub fn new(start: Wire, count: WireCount) -> Self {
        Self { start, count }
    }

    /// The first wire index in this range.
    #[must_use]
    pub fn start(&self) -> Wire {
        self.start
    }

    /// The number of wires in this range.
    #[must_use]
    pub fn count(&self) -> WireCount {
        self.count
    }

    /// Get the wire at a given offset within this range.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WireOutOfBounds`] if `offset >= count`.
    pub fn wire_at(&self, offset: usize) -> Result<Wire, Error> {
        if offset < self.count.0 {
            Ok(Wire::new(self.start.0 + offset))
        } else {
            Err(Error::WireOutOfBounds {
                wire_index: self.start.0 + offset,
                allocated: self.start.0 + self.count.0,
            })
        }
    }
}

/// Wire allocation state: tracks the next available wire index.
///
/// Threaded through interpretation as an immutable value,
/// producing new states via functional update.
#[derive(Debug, Clone, Copy)]
pub struct WireAllocator {
    next: usize,
}

impl WireAllocator {
    /// A fresh allocator starting at wire index 0.
    #[must_use]
    pub fn new() -> Self {
        Self { next: 0 }
    }

    /// Allocate a block of wires, returning the range and a new allocator.
    #[must_use]
    pub fn allocate(self, count: WireCount) -> (WireRange, Self) {
        let range = WireRange::new(Wire::new(self.next), count);
        let next_alloc = Self {
            next: self.next + count.count(),
        };
        (range, next_alloc)
    }

    /// The total number of wires allocated so far.
    #[must_use]
    pub fn total_allocated(self) -> usize {
        self.next
    }
}

impl Default for WireAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_produces_non_overlapping_ranges() {
        let alloc = WireAllocator::new();
        let (r1, alloc) = alloc.allocate(WireCount::new(3));
        let (r2, _alloc) = alloc.allocate(WireCount::new(2));
        assert_eq!(r1.start(), Wire::new(0));
        assert_eq!(r1.count(), WireCount::new(3));
        assert_eq!(r2.start(), Wire::new(3));
        assert_eq!(r2.count(), WireCount::new(2));
    }

    #[test]
    fn wire_at_in_bounds() -> Result<(), Error> {
        let range = WireRange::new(Wire::new(5), WireCount::new(3));
        assert_eq!(range.wire_at(0)?, Wire::new(5));
        assert_eq!(range.wire_at(1)?, Wire::new(6));
        assert_eq!(range.wire_at(2)?, Wire::new(7));
        Ok(())
    }

    #[test]
    fn wire_at_out_of_bounds() {
        let range = WireRange::new(Wire::new(0), WireCount::new(2));
        assert!(range.wire_at(2).is_err());
    }

    #[test]
    fn wire_count_tensor_is_addition() {
        let a = WireCount::new(3);
        let b = WireCount::new(4);
        assert_eq!(a.tensor(b), WireCount::new(7));
        assert_eq!(a + b, WireCount::new(7));
    }
}
