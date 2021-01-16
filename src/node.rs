use std::{
    cell::{Ref, RefCell},
    fmt,
    ops::Deref,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Colour {
    Black,
    Red,
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Colour::*;
        match self {
            Black => write!(f, "black"),
            Red => write!(f, "red"),
        }
    }
}

struct NodeData<T> {
    colour: Colour,
    parent: Option<ParentNode<T>>,
    left: Option<Node<T>>,
    right: Option<Node<T>>,
    pub(crate) data: T,
}

impl<T> NodeData<T> {
    fn new(data: T) -> NodeData<T> {
        NodeData {
            colour: Colour::Red,
            parent: None,
            left: None,
            right: None,
            data,
        }
    }
}

struct ParentNode<T>(Weak<RefCell<NodeData<T>>>);

/// Type of the tree elements containing the actuel data.
pub struct Node<T>(Rc<RefCell<NodeData<T>>>);

impl<T> Node<T> {
    pub(crate) fn id(&self) -> String {
        let address = format!("{:?}", self.0.as_ptr());
        address[2..].to_owned()
    }

    pub(crate) fn duplicate(&self) -> Node<T> {
        Node(Rc::clone(&self.0))
    }

    pub(crate) fn set_data(&mut self, data: T) {
        self.0.borrow_mut().data = data;
    }

    pub(crate) fn swap_data(&mut self, other: &mut Node<T>) {
        std::mem::swap(
            &mut self.0.borrow_mut().data,
            &mut other.0.borrow_mut().data,
        )
    }

    pub(crate) fn parent(&self) -> Option<Node<T>> {
        Some(Node(self.0.borrow().parent.as_ref()?.0.upgrade()?))
    }

    pub(crate) fn set_parent<I>(&mut self, node: I)
    where
        I: Into<Option<Node<T>>>,
    {
        self.0.borrow_mut().parent = node.into().map(|n| ParentNode(Rc::downgrade(&n.0)))
    }

    pub(crate) fn left(&self) -> Option<Node<T>> {
        self.0.borrow().left.as_ref().map(Node::duplicate)
    }

    pub(crate) fn set_left<I>(&mut self, node: I)
    where
        I: Into<Option<Node<T>>>,
    {
        self.0.borrow_mut().left = node.into()
    }

    pub(crate) fn right(&self) -> Option<Node<T>> {
        self.0.borrow().right.as_ref().map(Node::duplicate)
    }

    pub(crate) fn set_right<I>(&mut self, node: I)
    where
        I: Into<Option<Node<T>>>,
    {
        self.0.borrow_mut().right = node.into()
    }

    pub(crate) fn colour(&self) -> Colour {
        self.0.borrow().colour
    }

    pub(crate) fn set_colour(&mut self, colour: Colour) {
        self.0.borrow_mut().colour = colour;
    }

    /// Mutates the contained data in-place by applying the given closure.
    pub fn apply<F>(&self, f: F)
    where
        F: Fn(&mut T),
    {
        f(&mut self.0.borrow_mut().data);
    }

    /// Returns a reference to the contained data.
    pub fn data(&self) -> impl Deref<Target = T> + '_ {
        Ref::map(self.0.borrow(), |nd| &nd.data)
    }

    /// Returns a clone of the contained data.
    pub fn clone_data(&self) -> T
    where
        T: Clone,
    {
        self.0.borrow().data.clone()
    }
}

impl<T: Ord> Node<T> {
    pub(crate) fn is_left_child(&self) -> bool {
        self.parent()
            .as_ref()
            .and_then(Node::left)
            .as_ref()
            .map(|n| n == self)
            .unwrap_or(false)
    }

    pub(crate) fn sibling(&self) -> Option<Node<T>> {
        if self.is_left_child() {
            self.parent()?.right()
        } else {
            self.parent()?.left()
        }
    }

    pub(crate) fn uncle(&self) -> Option<Node<T>> {
        self.parent()?.sibling()
    }
}

impl<T> From<T> for Node<T> {
    fn from(data: T) -> Node<T> {
        Node(Rc::new(RefCell::new(NodeData::new(data))))
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node {{id: {}, p: {:?}, l: {:?}, r: {:?}, data: \"{:?}\"}}",
            self.id(),
            self.parent().as_ref().map(Node::id),
            self.left().as_ref().map(Node::id),
            self.right().as_ref().map(Node::id),
            self.0.borrow().data,
        )
    }
}

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
