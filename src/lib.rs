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

mod castset;
pub use castset::*;
