use std::{
    cell::{Ref, RefCell},
    fmt,
    ops::Deref,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Colour {
    Black,
    Red,
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Colour::Black => write!(f, "black"),
            Colour::Red => write!(f, "red"),
        }
    }
}

pub(crate) const NULL: u32 = u32::MAX;

pub(crate) struct NodeData<T> {
    pub(crate) colour: Colour,
    pub(crate) parent: u32,
    pub(crate) left: u32,
    pub(crate) right: u32,
    pub(crate) data: Option<T>,
}

impl<T> NodeData<T> {
    pub(crate) fn new(data: T) -> Self {
        NodeData {
            colour: Colour::Red,
            parent: NULL,
            left: NULL,
            right: NULL,
            data: Some(data),
        }
    }
}

pub(crate) type Arena<T> = Rc<RefCell<Vec<NodeData<T>>>>;

/// Type of the tree elements containing the actual data.
pub struct Node<T> {
    pub(crate) arena: Arena<T>,
    pub(crate) idx: u32,
}

impl<T> Node<T> {
    pub(crate) fn new(arena: Arena<T>, idx: u32) -> Self {
        Node { arena, idx }
    }

    pub(crate) fn duplicate(&self) -> Self {
        Node { arena: Rc::clone(&self.arena), idx: self.idx }
    }

    pub(crate) fn id(&self) -> String {
        self.idx.to_string()
    }

    pub(crate) fn colour(&self) -> Colour {
        self.arena.borrow()[self.idx as usize].colour
    }

    pub(crate) fn set_colour(&mut self, colour: Colour) {
        self.arena.borrow_mut()[self.idx as usize].colour = colour;
    }

    pub(crate) fn parent(&self) -> Option<Node<T>> {
        let idx = self.arena.borrow()[self.idx as usize].parent;
        if idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), idx)) }
    }

    pub(crate) fn set_parent<I: Into<Option<Node<T>>>>(&mut self, node: I) {
        let idx = node.into().map_or(NULL, |n| n.idx);
        self.arena.borrow_mut()[self.idx as usize].parent = idx;
    }

    pub(crate) fn left(&self) -> Option<Node<T>> {
        let idx = self.arena.borrow()[self.idx as usize].left;
        if idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), idx)) }
    }

    pub(crate) fn set_left<I: Into<Option<Node<T>>>>(&mut self, node: I) {
        let idx = node.into().map_or(NULL, |n| n.idx);
        self.arena.borrow_mut()[self.idx as usize].left = idx;
    }

    pub(crate) fn right(&self) -> Option<Node<T>> {
        let idx = self.arena.borrow()[self.idx as usize].right;
        if idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), idx)) }
    }

    pub(crate) fn set_right<I: Into<Option<Node<T>>>>(&mut self, node: I) {
        let idx = node.into().map_or(NULL, |n| n.idx);
        self.arena.borrow_mut()[self.idx as usize].right = idx;
    }

    pub(crate) fn is_left_child(&self) -> bool {
        let arena = self.arena.borrow();
        let parent_idx = arena[self.idx as usize].parent;
        parent_idx != NULL && arena[parent_idx as usize].left == self.idx
    }

    pub(crate) fn sibling(&self) -> Option<Node<T>> {
        let arena = self.arena.borrow();
        let parent_idx = arena[self.idx as usize].parent;
        if parent_idx == NULL { return None; }
        let parent = &arena[parent_idx as usize];
        let sib_idx = if parent.left == self.idx { parent.right } else { parent.left };
        drop(arena);
        if sib_idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), sib_idx)) }
    }

    pub(crate) fn uncle(&self) -> Option<Node<T>> {
        let arena = self.arena.borrow();
        let parent_idx = arena[self.idx as usize].parent;
        if parent_idx == NULL { return None; }
        let gp_idx = arena[parent_idx as usize].parent;
        if gp_idx == NULL { return None; }
        let gp = &arena[gp_idx as usize];
        let uncle_idx = if gp.left == parent_idx { gp.right } else { gp.left };
        drop(arena);
        if uncle_idx == NULL { None } else { Some(Node::new(Rc::clone(&self.arena), uncle_idx)) }
    }

    pub(crate) fn swap_data(&mut self, other: &mut Node<T>) {
        let mut arena = self.arena.borrow_mut();
        let a = self.idx as usize;
        let b = other.idx as usize;
        if a < b {
            let (lo, hi) = arena.split_at_mut(b);
            std::mem::swap(&mut lo[a].data, &mut hi[0].data);
        } else {
            let (lo, hi) = arena.split_at_mut(a);
            std::mem::swap(&mut lo[b].data, &mut hi[0].data);
        }
    }

    pub(crate) fn set_data(&mut self, data: T) {
        self.arena.borrow_mut()[self.idx as usize].data = Some(data);
    }

    /// Mutates the contained data in-place by applying the given closure.
    pub fn apply<F: Fn(&mut T)>(&self, f: F) {
        f(self.arena.borrow_mut()[self.idx as usize].data.as_mut().unwrap());
    }

    /// Returns a reference to the contained data.
    pub fn data(&self) -> impl Deref<Target = T> + '_ {
        Ref::map(self.arena.borrow(), |v| v[self.idx as usize].data.as_ref().unwrap())
    }

    /// Returns a clone of the contained data.
    pub fn clone_data(&self) -> T
    where
        T: Clone,
    {
        self.arena.borrow()[self.idx as usize].data.as_ref().unwrap().clone()
    }
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arena = self.arena.borrow();
        let nd = &arena[self.idx as usize];
        write!(
            f,
            "Node {{id: {}, p: {:?}, l: {:?}, r: {:?}, data: \"{:?}\"}}",
            self.idx,
            if nd.parent == NULL { None } else { Some(nd.parent) },
            if nd.left == NULL { None } else { Some(nd.left) },
            if nd.right == NULL { None } else { Some(nd.right) },
            nd.data.as_ref(),
        )
    }
}

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        Rc::ptr_eq(&self.arena, &other.arena) && self.idx == other.idx
    }
}
