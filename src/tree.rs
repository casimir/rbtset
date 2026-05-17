use std::cmp::Ordering;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

use crate::node::{Arena, Colour, Node, NodeData, NULL};
use crate::Consecutive;

/// A set based on a RB-Tree for efficient operations.
///
/// This implementation tries to be equally efficient for search, insert and delete
/// operations. Being based on a RB-Tree performing any of these operations is
/// `O(log n)` for an `O(n)` space complexity.
///
/// It is possible to iterate on a specific part of the set providing a `Node`
/// reference using [iter_from] or [values_from] avoiding unnecessary operations.
///
/// This data structure is made with very long usage in mind, think loads of
/// operations, and allows to optimize inner complexity and memory usage on demand
/// providing the stored data implements [Consecutive]. See [repack] for more
/// informations.
///
/// Like most of set implementations an `RBTreeSet` keeps its element sorted at
/// any time.
///
/// [iter_from]: #method.iter_from
/// [values_from]: #method.values_from
/// [Consecutive]: trait.Consecutive.html
/// [repack]: #method.repack
///
/// # Examples
///
/// ```
/// use rbtset::RBTreeSet;
///
/// // Type inference lets us omit an explicit type signature (which
/// // would be `RBTreeSet<isize>` in this example).
/// let mut numbers = RBTreeSet::new();
///
/// // Add some numbers.
/// numbers.insert(2);
/// numbers.insert(11);
/// numbers.insert(6);
/// numbers.insert(10);
/// numbers.insert(26);
/// numbers.insert(7);
/// numbers.insert(18);
/// numbers.insert(8);
/// numbers.insert(13);
/// numbers.insert(22);
///
/// // Check for a specific one.
/// if numbers.get(&101).is_none() {
///     println!("We have {} numbers, but 101 ain't one.", numbers.len());
/// }
///
/// // Remove a number.
/// numbers.remove(&26);
///
/// // Iterate over everything.
/// for number in numbers.values() {
///     println!("{}", number);
/// }
/// ```
pub struct RBTreeSet<T> {
    arena: Arena<T>,
    free: Vec<u32>,
    root: u32,
    length: usize,
}

impl<T: Ord> Default for RBTreeSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> RBTreeSet<T> {
    /// Makes a new `RBTreeSet`.
    pub fn new() -> RBTreeSet<T> {
        RBTreeSet {
            arena: Rc::new(std::cell::RefCell::new(Vec::new())),
            free: Vec::new(),
            root: NULL,
            length: 0,
        }
    }

    fn node_opt(&self, idx: u32) -> Option<Node<T>> {
        if idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), idx)) }
    }

    fn alloc_node(&mut self, data: T) -> Node<T> {
        let idx = if let Some(idx) = self.free.pop() {
            self.arena.borrow_mut()[idx as usize] = NodeData::new(data);
            idx
        } else {
            let mut arena = self.arena.borrow_mut();
            let idx = arena.len() as u32;
            arena.push(NodeData::new(data));
            idx
        };
        Node::new(Rc::clone(&self.arena), idx)
    }

    fn free_node(&mut self, idx: u32) {
        {
            let mut arena = self.arena.borrow_mut();
            let nd = &mut arena[idx as usize];
            nd.data = None;
            nd.left = NULL;
            nd.right = NULL;
            nd.parent = NULL;
        }
        self.free.push(idx);
    }

    /// Returns the value in the set, if any, that is matching the given value.
    ///
    /// Use [get_node] in pair with [Node::data] if you want to avoid value cloning.
    ///
    /// [get_node]: #method.get_node
    /// [Node::data]: struct.Node.html#method.data
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.get(&2), Some(2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    pub fn get(&self, data: &T) -> Option<T>
    where
        T: Clone,
    {
        self.get_node(data).as_ref().map(|n| n.clone_data())
    }

    fn insert_from(&mut self, root_idx: u32, data: T) -> Option<Node<T>> {
        let ord = {
            let arena = self.arena.borrow();
            data.cmp(arena[root_idx as usize].data.as_ref().unwrap())
        };
        match ord {
            Ordering::Equal => None,
            Ordering::Less => {
                let left_idx = self.arena.borrow()[root_idx as usize].left;
                if left_idx == NULL {
                    let mut node = self.alloc_node(data);
                    node.set_parent(Node::new(Rc::clone(&self.arena), root_idx));
                    self.arena.borrow_mut()[root_idx as usize].left = node.idx;
                    Some(node)
                } else {
                    self.insert_from(left_idx, data)
                }
            }
            Ordering::Greater => {
                let right_idx = self.arena.borrow()[root_idx as usize].right;
                if right_idx == NULL {
                    let mut node = self.alloc_node(data);
                    node.set_parent(Node::new(Rc::clone(&self.arena), root_idx));
                    self.arena.borrow_mut()[root_idx as usize].right = node.idx;
                    Some(node)
                } else {
                    self.insert_from(right_idx, data)
                }
            }
        }
    }

    fn rotate_right(&mut self, mut node: Node<T>) {
        let mut parent = node.left().expect("get parent node");
        node.set_left(parent.right());
        if let Some(mut right) = parent.right() {
            right.set_parent(node.duplicate());
        }
        parent.set_right(node.duplicate());
        parent.set_parent(node.parent());
        if let Some(mut gparent) = parent.parent() {
            if node.is_left_child() {
                gparent.set_left(parent.duplicate());
            } else {
                gparent.set_right(parent.duplicate());
            }
        } else {
            self.root = parent.idx;
        }
        node.set_parent(parent);
    }

    fn rotate_left(&mut self, mut node: Node<T>) {
        let mut parent = node.right().expect("get parent node");
        node.set_right(parent.left());
        if let Some(mut left) = parent.left() {
            left.set_parent(node.duplicate());
        }
        parent.set_left(node.duplicate());
        parent.set_parent(node.parent());
        if let Some(mut gparent) = parent.parent() {
            if node.is_left_child() {
                gparent.set_left(parent.duplicate());
            } else {
                gparent.set_right(parent.duplicate());
            }
        } else {
            self.root = parent.idx;
        }
        node.set_parent(parent);
    }

    fn balance(&mut self, mut node: Node<T>) {
        let mut parent = match node.parent() {
            None => {
                node.set_colour(Colour::Black);
                return;
            }
            Some(p) => p,
        };

        if parent.colour() == Colour::Black {
            return;
        }

        let mut uncle = node.uncle();
        if uncle.as_ref().map(Node::colour) == Some(Colour::Red) {
            parent.set_colour(Colour::Black);
            uncle.as_mut().unwrap().set_colour(Colour::Black);
            let mut grand_parent = parent.parent().unwrap();
            grand_parent.set_colour(Colour::Red);
            self.balance(grand_parent.duplicate());
        } else {
            let mut new_node = node.duplicate();

            let parent_is_left = parent.is_left_child();
            let node_is_left = node.is_left_child();
            if parent_is_left && !node_is_left {
                self.rotate_left(parent.duplicate());
                new_node = node.left().unwrap();
            } else if !parent_is_left && node_is_left {
                self.rotate_right(parent.duplicate());
                new_node = node.right().unwrap();
            }

            let mut new_parent = new_node.parent().unwrap();
            let mut new_gparent = new_parent.parent().unwrap();
            new_parent.set_colour(Colour::Black);
            new_gparent.set_colour(Colour::Red);

            if new_node.is_left_child() {
                self.rotate_right(new_gparent.duplicate());
            } else {
                self.rotate_left(new_gparent.duplicate());
            }
        }
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have a matching value present, the new node is returned.
    ///
    /// If the set did have a matching value present, None is returned, and the entry is not updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut set = RBTreeSet::new();
    /// assert!(set.insert(2).is_some());
    /// assert!(set.insert(2).is_none());
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, data: T) -> Option<Node<T>> {
        let node = if self.root == NULL {
            let node = self.alloc_node(data);
            self.root = node.idx;
            Some(node)
        } else {
            self.insert_from(self.root, data)
        };
        if let Some(ref n) = node {
            self.balance(n.duplicate());
            self.length += 1;
        }
        node
    }

    /// Removes a matching value from the set. Returns whether a matching value was present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut set = RBTreeSet::new();
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    pub fn remove(&mut self, data: &T) -> bool {
        match self.get_node(data) {
            Some(ref mut node) => {
                self.remove_node(node);
                true
            }
            None => false,
        }
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut v = RBTreeSet::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.arena = Rc::new(std::cell::RefCell::new(Vec::new()));
        self.free.clear();
        self.root = NULL;
        self.length = 0;
    }

    /// Returns the node in the set, if any, that is matching the given value.
    ///
    /// If the value is cheap to clone [get] should be more convenient to use.
    ///
    /// [get]: #method.get
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(*set.get_node(&2).unwrap().data(), 2);
    /// assert_eq!(set.get_node(&4), None);
    /// ```
    pub fn get_node(&self, data: &T) -> Option<Node<T>> {
        let mut idx = self.root;
        while idx != NULL {
            let arena = self.arena.borrow();
            let ord = data.cmp(arena[idx as usize].data.as_ref().unwrap());
            let next = match ord {
                Ordering::Equal => return Some(Node::new(Rc::clone(&self.arena), idx)),
                Ordering::Less => arena[idx as usize].left,
                Ordering::Greater => arena[idx as usize].right,
            };
            drop(arena);
            idx = next;
        }
        None
    }

    fn successor(node: Node<T>) -> Option<Node<T>> {
        if let Some(right) = node.right() {
            let mut tmp = right;
            while let Some(n) = tmp.left() {
                tmp = n;
            }
            Some(tmp)
        } else if node.is_left_child() {
            node.parent()
        } else {
            let mut tmp = node;
            loop {
                let parent = tmp.parent()?;
                if parent.left().as_ref() == Some(&tmp) {
                    return Some(parent);
                }
                tmp = parent;
            }
        }
    }

    fn double_black_fixup(&mut self, node: &Node<T>) {
        if self.root == node.idx {
            return;
        }

        let mut parent = node.parent().unwrap();
        if let Some(ref mut sibling) = node.sibling() {
            if sibling.colour() == Colour::Red {
                parent.set_colour(Colour::Red);
                sibling.set_colour(Colour::Black);
                if sibling.is_left_child() {
                    self.rotate_right(parent);
                } else {
                    self.rotate_left(parent);
                }
                self.double_black_fixup(node);
            } else if sibling.left().as_ref().map(Node::colour) == Some(Colour::Red)
                || sibling.right().as_ref().map(Node::colour) == Some(Colour::Red)
            {
                if sibling.left().as_ref().map(Node::colour) == Some(Colour::Red) {
                    let mut left = sibling.left().unwrap();
                    if sibling.is_left_child() {
                        left.set_colour(sibling.colour());
                        sibling.set_colour(parent.colour());
                        self.rotate_right(parent.duplicate());
                    } else {
                        left.set_colour(parent.colour());
                        self.rotate_right(sibling.duplicate());
                        self.rotate_left(parent.duplicate());
                    }
                } else {
                    let mut right = sibling.right().unwrap();
                    if sibling.is_left_child() {
                        right.set_colour(parent.colour());
                        self.rotate_left(sibling.duplicate());
                        self.rotate_right(parent.duplicate());
                    } else {
                        right.set_colour(sibling.colour());
                        sibling.set_colour(parent.colour());
                        self.rotate_left(parent.duplicate());
                    }
                }
                parent.set_colour(Colour::Black);
            } else {
                sibling.set_colour(Colour::Red);
                if parent.colour() == Colour::Black {
                    self.double_black_fixup(&parent);
                } else {
                    parent.set_colour(Colour::Black);
                }
            }
        } else {
            self.double_black_fixup(&parent)
        }
    }

    /// Removes a node from the set. This method expects a matching node to be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut set = RBTreeSet::new();
    /// set.insert(2);
    /// let mut node = set.get_node(&2).unwrap();
    /// set.remove_node(&mut node);
    /// assert!(set.is_empty());
    /// ```
    pub fn remove_node(&mut self, node: &mut Node<T>) {
        let new_node = if node.left().is_some() && node.right().is_some() {
            Self::successor(node.duplicate())
        } else if node.left().is_some() {
            node.left()
        } else {
            node.right()
        };
        let double_black = node.colour() == Colour::Black
            && new_node.as_ref().map(Node::colour) != Some(Colour::Red);

        if new_node.is_none() {
            if self.root == node.idx {
                self.root = NULL;
            } else {
                if double_black {
                    self.double_black_fixup(node)
                } else if let Some(mut sibling) = node.sibling() {
                    sibling.set_colour(Colour::Red);
                }

                if let Some(mut parent) = node.parent() {
                    if node.is_left_child() {
                        parent.set_left(None);
                    } else {
                        parent.set_right(None);
                    }
                }
            }
            self.length -= 1;
            self.free_node(node.idx);
            return;
        }

        let mut substitute = new_node.unwrap();

        if node.left().is_none() || node.right().is_none() {
            if self.root == node.idx {
                node.swap_data(&mut substitute);
                node.set_left(None);
                node.set_right(None);
                self.free_node(substitute.idx);
            } else if let Some(mut parent) = node.parent() {
                if node.is_left_child() {
                    parent.set_left(substitute.duplicate());
                } else {
                    parent.set_right(substitute.duplicate());
                }
                substitute.set_parent(node.parent());
                if double_black {
                    self.double_black_fixup(&substitute);
                } else {
                    substitute.set_colour(Colour::Black)
                }
                self.free_node(node.idx);
            }
            self.length -= 1;
            return;
        }

        substitute.swap_data(node);
        self.remove_node(&mut substitute);
    }

    /// Returns the first node of the set if not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut set: RBTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(*set.first().unwrap().data(), 1);
    /// set.clear();
    /// assert_eq!(set.first(), None);
    /// ```
    pub fn first(&self) -> Option<Node<T>> {
        let mut n = self.node_opt(self.root)?;
        while let Some(left) = n.left() {
            n = left;
        }
        Some(n)
    }

    /// Returns the last node of the set if not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut set: RBTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(*set.last().unwrap().data(), 3);
    /// set.clear();
    /// assert_eq!(set.last(), None);
    /// ```
    pub fn last(&self) -> Option<Node<T>> {
        let mut n = self.node_opt(self.root)?;
        while let Some(right) = n.right() {
            n = right;
        }
        Some(n)
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut v = RBTreeSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns true if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let mut v = RBTreeSet::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Gets an iterator that visits the nodes in the RBTreeSet in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [3, 1, 2].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    ///
    /// assert_eq!(*set_iter.next().unwrap().data(), 1);
    /// assert_eq!(*set_iter.next().unwrap().data(), 2);
    /// assert_eq!(*set_iter.next().unwrap().data(), 3);
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter { cursor: self.first() }
    }

    /// Gets an iterator that visits the nodes in the RBTreeSet in ascending order,
    /// starting at the given node.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [3, 1, 2].iter().cloned().collect();
    /// let node = set.get_node(&2).unwrap();
    /// let mut set_iter = set.iter_from(&node);
    ///
    /// assert_eq!(*set_iter.next().unwrap().data(), 2);
    /// assert_eq!(*set_iter.next().unwrap().data(), 3);
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter_from(&self, node: &Node<T>) -> Iter<T> {
        Iter { cursor: Some(node.duplicate()) }
    }

    /// Gets an iterator that visit the nodes values in the RBTreeSet in ascending order.
    ///
    /// This iterator clones the values. Use [iter] in pair with [Node::data] if you want
    /// to avoid the cloning.
    ///
    /// [iter]: #method.iter
    /// [Node::data]: struct.Node.html#method.data
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [3, 1, 2].iter().cloned().collect();
    /// let mut set_values = set.values();
    ///
    /// assert_eq!(set_values.next(), Some(1));
    /// assert_eq!(set_values.next(), Some(2));
    /// assert_eq!(set_values.next(), Some(3));
    /// assert_eq!(set_values.next(), None);
    /// ```
    pub fn values(&self) -> IterValues<T>
    where
        T: Clone,
    {
        IterValues { inner: self.iter() }
    }

    /// Gets an iterator that visit the nodes values in the RBTreeSet in ascending order,
    /// starting at the given node.
    ///
    /// This iterator clones the values. Use [iter_from] in pair with [Node::data] if you want
    /// to avoid the cloning.
    ///
    /// [iter_from]: #method.iter_from
    /// [Node::data]: struct.Node.html#method.data
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = [3, 1, 2].iter().cloned().collect();
    /// let node = set.get_node(&2).unwrap();
    /// let mut set_values = set.values_from(&node);
    ///
    /// assert_eq!(set_values.next(), Some(2));
    /// assert_eq!(set_values.next(), Some(3));
    /// assert_eq!(set_values.next(), None);
    /// ```
    pub fn values_from(&self, node: &Node<T>) -> IterValues<T>
    where
        T: Clone,
    {
        IterValues { inner: self.iter_from(node) }
    }

    /// Optimize the set by merging nodes where applicable while keeping the ordering.
    ///
    /// Two nodes can be merged together when [consecutive].
    ///
    /// [consecutive]: trait.Consecutive.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use rbtset::{Consecutive, RBTreeSet};
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
    /// let mut set = RBTreeSet::new();
    /// set.insert(Seq(1..3));
    /// set.insert(Seq(5..8));
    /// set.insert(Seq(8..13));
    /// set.insert(Seq(13..16));
    /// set.insert(Seq(23..26));
    ///
    /// assert_eq!(
    ///     set.values().collect::<Vec<Seq>>(),
    ///     vec![Seq(1..3), Seq(5..8), Seq(8..13), Seq(13..16), Seq(23..26)]
    /// );
    /// set.repack();
    /// assert_eq!(
    ///     set.values().collect::<Vec<Seq>>(),
    ///     vec![Seq(1..3), Seq(5..16), Seq(23..26)]
    /// );
    /// ```
    pub fn repack(&mut self)
    where
        T: Clone + Consecutive,
    {
        if self.is_empty() {
            return;
        }
        let mut nodes = self.iter();
        let mut prev = nodes.next().unwrap();
        while let Some(curr) = nodes.next() {
            let mut acc = vec![prev.clone_data()];
            let mut cursor = curr;
            while prev.data().consecutive(&cursor.data()) {
                acc.push(cursor.clone_data());
                prev = cursor.duplicate();
                if let Some(n) = nodes.next() {
                    cursor = n;
                } else {
                    break;
                }
            }
            if acc.len() > 1 {
                let new_data = acc.iter().skip(1).fold(acc[0].clone(), |a, b| a.merged(b));
                for data in &acc[0..acc.len() - 1] {
                    self.remove(data);
                }
                let last = &acc[acc.len() - 1];
                let mut last_node = self.get_node(last).expect("get node");
                last_node.set_data(new_data);
            }
            prev = cursor;
        }
    }

    /// Returns the serialization of the set as an RB-tree in DOT.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbtset::RBTreeSet;
    ///
    /// let set: RBTreeSet<_> = vec![2, 11, 22, 7].iter().cloned().collect();
    /// print!("{}", set.dump_tree_as_dot());
    /// ```
    pub fn dump_tree_as_dot(&self) -> String
    where
        T: fmt::Debug,
    {
        let mut lines = Vec::new();
        lines.push(String::from("graph RBTreeSet {"));

        let mut definitions = Vec::new();
        let mut links = Vec::new();
        let mut tmp = self.first();
        while let Some(ref node) = tmp {
            definitions.push(format!(
                "    Node{} [label=\"{:?}\", color={}]",
                node.id(),
                *node.data(),
                node.colour()
            ));
            if node.left().is_some() {
                links.push(format!(
                    "    Node{} -- Node{}",
                    node.id(),
                    node.left().as_ref().unwrap().id()
                ));
            } else {
                definitions.push(format!("    NullL{} [shape=point]", node.id()));
                links.push(format!("    Node{0} -- NullL{0}", node.id()));
            }
            if node.right().is_some() {
                links.push(format!(
                    "    Node{} -- Node{}",
                    node.id(),
                    node.right().as_ref().unwrap().id()
                ));
            } else {
                definitions.push(format!("    NullR{} [shape=point]", node.id()));
                links.push(format!("    Node{0} -- NullR{0}", node.id()));
            }
            tmp = Self::successor(node.duplicate());
        }

        lines.append(&mut definitions);
        lines.push(String::new());
        lines.append(&mut links);

        lines.push(String::from("}"));
        lines.push(String::new());
        lines.join("\n")
    }
}

impl<T> fmt::Debug for RBTreeSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RBTreeSet {{ length: {} }}", self.length)
    }
}

impl<T: Clone> Clone for RBTreeSet<T> {
    fn clone(&self) -> Self {
        let arena_clone: Vec<NodeData<T>> = self.arena.borrow().iter().map(|nd| NodeData {
            colour: nd.colour,
            parent: nd.parent,
            left: nd.left,
            right: nd.right,
            data: nd.data.clone(),
        }).collect();
        RBTreeSet {
            arena: Rc::new(std::cell::RefCell::new(arena_clone)),
            free: self.free.clone(),
            root: self.root,
            length: self.length,
        }
    }
}

/// Created with the method [iter] or with [iter_from] for partial iterations.
///
/// [iter]: struct.RBTreeSet.html#method.iter
/// [iter_from]: struct.RBTreeSet.html#method.iter_from
pub struct Iter<T> {
    cursor: Option<Node<T>>,
}

impl<T: Ord> Iterator for Iter<T> {
    type Item = Node<T>;

    fn next(&mut self) -> Option<Node<T>> {
        let node = self.cursor.as_ref().map(Node::duplicate);
        if let Some(ref n) = self.cursor {
            self.cursor = RBTreeSet::successor(n.duplicate());
        }
        node
    }
}

/// Created with the method [values] or with [values_from] for partial iterations.
///
/// [values]: struct.RBTreeSet.html#method.values
/// [values_from]: struct.RBTreeSet.html#method.values_from
pub struct IterValues<T> {
    inner: Iter<T>,
}

impl<T: Clone + Ord> Iterator for IterValues<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.next().as_ref().map(Node::clone_data)
    }
}

impl<T: Ord> FromIterator<T> for RBTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut s = Self::new();
        for i in iter {
            s.insert(i);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    macro_rules! assert_node {
        ($node:expr, NULL) => {
            assert!(node.is_none())
        };
        ($node:expr, $data:expr) => {
            assert_eq!($node.as_ref().unwrap().borrow().data, $data);
        };
        ($node:expr, $data:expr, $colour:expr) => {
            assert_eq!(*$node.as_ref().unwrap().data(), $data);
            assert_eq!($node.as_ref().unwrap().colour(), $colour);
        };
    }

    fn root_node<T: Ord>(tree: &RBTreeSet<T>) -> Option<Node<T>> {
        tree.node_opt(tree.root)
    }

    #[test]
    fn rotate_left_root() {
        let mut tree = RBTreeSet::new();
        tree.insert(2);
        tree.insert(11);
        tree.insert(15);

        print!("{}", tree.dump_tree_as_dot());
        assert_node!(root_node(&tree), 11, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().left(), 2, Colour::Red);
        assert_node!(root_node(&tree).as_ref().unwrap().right(), 15, Colour::Red);
    }

    #[test]
    fn rotate_left_parent() {
        let mut tree = RBTreeSet::new();
        tree.insert(3);
        tree.insert(6);
        tree.insert(2);
        tree.insert(11);
        tree.insert(15);

        print!("{}", tree.dump_tree_as_dot());
        assert_node!(root_node(&tree), 3, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().left(), 2, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().right(), 11, Colour::Black);
    }

    #[test]
    fn rotate_right_root() {
        let mut tree = RBTreeSet::new();
        tree.insert(11);
        tree.insert(6);
        tree.insert(2);

        print!("{}", tree.dump_tree_as_dot());
        assert_node!(root_node(&tree), 6, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().left(), 2, Colour::Red);
        assert_node!(root_node(&tree).as_ref().unwrap().right(), 11, Colour::Red);
    }

    #[test]
    fn rotate_right_parent() {
        let mut tree = RBTreeSet::new();
        tree.insert(11);
        tree.insert(6);
        tree.insert(15);
        tree.insert(3);
        tree.insert(2);

        print!("{}", tree.dump_tree_as_dot());
        assert_node!(root_node(&tree), 11, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().left(), 3, Colour::Black);
        assert_node!(root_node(&tree).as_ref().unwrap().right(), 15, Colour::Black);
    }

    #[derive(Debug)]
    enum InvalidReason<T> {
        RootIsRed,
        RedHasRedChild(T),
        InvalidDepth(T, i64),
    }

    fn validate_subtree<T>(
        node: &Node<T>,
        leaves: &mut Vec<Node<T>>,
    ) -> Result<(), InvalidReason<T>>
    where
        T: Clone + fmt::Debug + Ord,
    {
        if node.colour() == Colour::Red
            && (node.left().as_ref().map(Node::colour) == Some(Colour::Red)
                || node.right().as_ref().map(Node::colour) == Some(Colour::Red))
        {
            Err(InvalidReason::RedHasRedChild(node.clone_data()))
        } else {
            if let Some(ref n) = node.left() {
                validate_subtree(n, leaves)?;
            }
            if let Some(ref n) = node.right() {
                validate_subtree(n, leaves)?;
            }
            if node.left().is_none() || node.right().is_none() {
                leaves.push(node.duplicate());
            }
            Ok(())
        }
    }

    fn validate_tree<T>(tree: &RBTreeSet<T>) -> Result<(), InvalidReason<T>>
    where
        T: Clone + fmt::Debug + Ord,
    {
        if let Some(ref root) = root_node(tree) {
            if root.colour() == Colour::Red {
                Err(InvalidReason::RootIsRed)
            } else {
                let mut leaves = Vec::new();
                validate_subtree(root, &mut leaves)?;

                let mut black_height = 0;
                for n in &leaves {
                    let mut tmp = Some(n.duplicate());
                    let mut leave_depth = 0;
                    while let Some(ref n) = tmp {
                        if n.colour() == Colour::Black {
                            leave_depth += 1;
                        }
                        tmp = n.parent();
                    }
                    if black_height == 0 {
                        black_height = leave_depth;
                    } else if leave_depth != black_height {
                        return Err(InvalidReason::InvalidDepth(
                            n.clone_data(),
                            leave_depth - black_height,
                        ));
                    }
                }
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    #[test]
    fn iterator() {
        let mut set = RBTreeSet::new();
        set.insert(2);
        set.insert(11);
        set.insert(6);
        set.insert(10);
        set.insert(26);
        set.insert(7);
        set.insert(18);
        set.insert(8);
        set.insert(13);
        set.insert(22);

        assert_eq!(
            set.values().collect::<Vec<i32>>(),
            vec![2, 6, 7, 8, 10, 11, 13, 18, 22, 26]
        );
    }

    #[test]
    fn insert() {
        let mut set = RBTreeSet::new();
        set.insert(2);
        set.insert(11);
        set.insert(6);
        set.insert(10);
        set.insert(26);
        set.insert(7);
        set.insert(18);
        set.insert(8);
        set.insert(13);
        set.insert(22);

        print!("{}", set.dump_tree_as_dot());
        validate_tree(&set).expect("validate tree");
        assert_eq!(
            set.values().collect::<Vec<i32>>(),
            vec![2, 6, 7, 8, 10, 11, 13, 18, 22, 26]
        );
        assert_eq!(set.len(), 10);
    }

    #[test]
    fn delete_pseudoleaves() {
        let mut set = RBTreeSet::new();
        set.insert(50);
        set.insert(20);
        set.insert(60);
        set.insert(30);
        set.insert(40);
        set.insert(70);
        set.insert(80);

        set.remove(&20);
        assert_eq!(
            set.values().collect::<Vec<i32>>(),
            vec![30, 40, 50, 60, 70, 80]
        );

        set.remove(&30);
        assert_eq!(set.values().collect::<Vec<i32>>(), vec![40, 50, 60, 70, 80]);

        set.remove(&80);
        assert_eq!(set.values().collect::<Vec<i32>>(), vec![40, 50, 60, 70]);

        set.remove(&70);
        print!("{}", set.dump_tree_as_dot());
        validate_tree(&set).expect("validate tree");
        assert_eq!(set.values().collect::<Vec<i32>>(), vec![40, 50, 60]);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn delete() {
        let mut keep = Vec::new();
        let mut remove = Vec::new();
        for i in (1..30).step_by(3) {
            keep.push(i);
            remove.push(i + 2);
        }

        let mut tree = RBTreeSet::new();
        for i in remove.iter().rev() {
            tree.insert(*i).expect("insert node");
        }
        for i in &keep {
            tree.insert(*i).expect("insert node");
        }
        for i in remove {
            assert!(tree.remove(&i));
        }

        print!("{}", tree.dump_tree_as_dot());
        validate_tree(&tree).expect("validate tree");
        assert_eq!(tree.values().collect::<Vec<i32>>(), keep);
        assert_eq!(tree.len(), keep.len());
    }

    #[test]
    fn clone() {
        let mut set = RBTreeSet::new();
        set.insert(50);
        set.insert(20);
        set.insert(60);
        set.insert(30);
        set.insert(40);
        set.insert(70);
        set.insert(80);
        let tree_bis = set.clone();

        assert_eq!(
            set.values().collect::<Vec<i32>>(),
            tree_bis.values().collect::<Vec<i32>>()
        );

        set.remove(&60);
        assert_eq!(set.len(), tree_bis.len() - 1);
    }

    #[derive(Debug, Clone, Eq)]
    struct Seq(std::ops::Range<usize>);

    impl Ord for Seq {
        fn cmp(&self, other: &Seq) -> Ordering {
            self.0.start.cmp(&other.0.start)
        }
    }

    impl PartialOrd for Seq {
        fn partial_cmp(&self, other: &Seq) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for Seq {
        fn eq(&self, other: &Seq) -> bool {
            other.0.start <= self.0.start && self.0.start < other.0.end
        }
    }

    impl Consecutive for Seq {
        fn consecutive(&self, other: &Seq) -> bool {
            self.0.end == other.0.start
        }

        fn merged(&self, other: &Seq) -> Seq {
            Seq(self.0.start..other.0.end)
        }
    }

    #[test]
    fn repack() {
        let mut set = RBTreeSet::new();
        set.insert(Seq(1..3));
        set.insert(Seq(5..8));
        set.insert(Seq(8..13));
        set.insert(Seq(13..16));
        set.insert(Seq(23..26));

        validate_tree(&set).expect("validate tree");
        assert_eq!(
            set.values().collect::<Vec<Seq>>(),
            vec![Seq(1..3), Seq(5..8), Seq(8..13), Seq(13..16), Seq(23..26)]
        );
        assert_eq!(set.len(), 5);

        set.repack();

        print!("{}", set.dump_tree_as_dot());
        validate_tree(&set).expect("validate tree");
        assert_eq!(
            set.values().collect::<Vec<Seq>>(),
            vec![Seq(1..3), Seq(5..16), Seq(23..26)]
        );
        assert_eq!(set.len(), 3);
    }
}
