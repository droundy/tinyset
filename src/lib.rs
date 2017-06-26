//! david-set is a collection that is efficient for small numbers of
//! elements, while still scaling well for large numbers.  It is
//! basically interchangeable with `HashSet`, although it requires
//! that its elements implement the `Copy` trait, since otherwise I
//! would have to learn to write correct `unsafe` code, which would be
//! scary.
//!
//! # Example
//!
//! ```
//! use david_set::Set;
//! let mut s: Set<usize> = Set::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```

#![deny(missing_docs)]

mod vecset;
pub use vecset::*;

mod copyset;
pub use copyset::*;

/// Trait for any type that can be converted to a `usize`.  This could
/// actually be a hash function, but we will assume that it is *fast*,
/// so I'm not calling it `Hash`.
pub trait Cast {
    /// Convert to a `usize`.
    fn cast(self) -> usize;
}

impl Cast for usize {
    fn cast(self) -> usize { self }
}
impl Cast for u64 {
    fn cast(self) -> usize { self as usize }
}
impl Cast for u32 {
    fn cast(self) -> usize { self as usize }
}
impl Cast for u16 {
    fn cast(self) -> usize { self as usize }
}
impl Cast for u8 {
    fn cast(self) -> usize { self as usize }
}

