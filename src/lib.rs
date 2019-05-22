//! A set based on a RB-Tree for efficient operations.

mod node;
mod tree;

pub use node::Node;
pub use tree::{Iter, IterValues, RBTreeSet};

/// An interface for dealing with consecutive data.
///
/// Provides a way to determine if two values are consecutive and how to merge
/// it. See [repack] for the main usage of this interface.
///
/// [repack]: struct.RBTreeSet.html#method.repack
///
/// # Examples
///
/// ```
/// use std::cmp::Ordering;
/// use rbtset::Consecutive;
///
/// #[derive(Debug, Clone, Eq)]
/// struct Seq(std::ops::Range<usize>);
///
/// impl Ord for Seq {
///     fn cmp(&self, other: &Seq) -> Ordering {
///         self.0.start.cmp(&other.0.start)
///     }
/// }
///
/// impl PartialOrd for Seq {
///     fn partial_cmp(&self, other: &Seq) -> Option<Ordering> {
///         Some(self.cmp(other))
///     }
/// }
///
/// impl PartialEq for Seq {
///     fn eq(&self, other: &Seq) -> bool {
///         other.0.start <= self.0.start && self.0.start < other.0.end
///     }
/// }
///
/// impl Consecutive for Seq {
///     fn consecutive(&self, other: &Seq) -> bool {
///         self.0.end == other.0.start
///     }
///
///     fn merged(&self, other: &Seq) -> Seq {
///         Seq(self.0.start..other.0.end)
///     }
/// }
///
/// assert_eq!(Seq(1..3).consecutive(&Seq(3..5)), true);
/// assert_eq!(Seq(1..3).consecutive(&Seq(4..5)), false);
///
/// assert_eq!(Seq(1..3).merged(&Seq(3..5)), Seq(1..5));
/// ```
pub trait Consecutive {
    /// Returns true if `other` is consecutive to `self`.
    fn consecutive(&self, other: &Self) -> bool;
    /// Returns the merge of `self` and `other`. The functions assumes that
    /// `other` is consecutive to `self`.
    fn merged(&self, other: &Self) -> Self;
}
