//! Sorting functionality for tree nodes.
//!
//! This module provides methods for sorting children of a node in a tree.
//! The sorting can be done based on the node values or their indices.

use std::cmp::Ordering;

use crate::{NodeId, NodeMut};

impl<'a, T: 'a> NodeMut<'a, T> {
    /// Sort children by value in ascending order.
    ///
    /// This method is a shorthand for calling `sort_by` with the `Ord::cmp` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    ///
    /// let mut tree = tree!('a' => { 'd', 'c', 'b' });
    /// tree.root_mut().sort();
    /// assert_eq!(
    ///     vec![&'b', &'c', &'d'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(|a, b| a.cmp(b));
    }

    /// Sort children by value in ascending order using a comparison function.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    ///
    /// let mut tree = tree!('a' => { 'c', 'd', 'b' });
    /// tree.root_mut().sort_by(|a, b| b.cmp(a));
    /// assert_eq!(
    ///     vec![&'d', &'c', &'b'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        if self.has_children() {
            let (unsorted, sorted) = self.sort_handler(|nodes| {
                nodes.sort_by(|(_, a), (_, b)| compare(a, b));
            });

            self.swap(unsorted, sorted);
        }
    }

    /// Sort children by value's key in ascending order using a key extraction function.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    ///
    /// let mut tree = tree!("1a" => { "2b", "4c", "3d" });
    /// tree.root_mut().sort_by_key(|a| a.split_at(1).0.parse::<i32>().unwrap());
    /// assert_eq!(
    ///     vec!["2b", "3d", "4c"],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| *n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        if self.has_children() {
            let (unsorted, sorted) = self.sort_handler(|nodes| {
                nodes.sort_by_key(|(_, value)| f(value));
            });

            self.swap(unsorted, sorted);
        }
    }

    /// Sort children by their NodeId in ascending order. The purpose is to restore the original order.
    ///
    /// This method is a shorthand for calling `sort_by_id` with the `Ord::cmp` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    ///
    /// let mut tree = tree!('a' => { 'd', 'c', 'b' });
    /// tree.root_mut().sort();
    /// assert_ne!(
    ///     vec![&'d', &'c', &'b'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// tree.root_mut().sort_id();
    /// assert_eq!(
    ///     vec![&'d', &'c', &'b'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort_id(&mut self) {
        self.sort_by_id(|a, b| a.cmp(&b));
    }

    /// Sort children by their NodeId's index using a comparison function.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    ///
    /// let mut tree = tree!('a' => { 'd', 'b', 'c' });
    /// tree.root_mut().sort_by_id(|a, b| b.cmp(&a));
    /// assert_eq!(
    ///     vec![&'c', &'b', &'d'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort_by_id<F>(&mut self, mut compare: F)
    where
        F: FnMut(usize, usize) -> Ordering,
    {
        if self.has_children() {
            let (unsorted, sorted) = self.sort_handler(|nodes| {
                nodes.sort_by(|(ida, _), (idb, _)| compare(ida.to_index(), idb.to_index()));
            });

            self.swap(unsorted, sorted);
        }
    }

    /// Sort children by a key function taking a NodeId's index and a `&T` reference 
    /// returning a key of type `K` that implements `Ord`.
    ///
    /// I don't know how to use this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use ego_tree::tree;
    /// let mut tree = tree!('a' => { 'd', 'b', 'c' });
    /// tree.root_mut()
    ///     .sort_by_id_key(|id, value| id + *value as usize); // {1+100, 2+98, 3+99}
    /// assert_eq!(
    ///     vec![&'b', &'d', &'c'],
    ///     tree.root()
    ///         .children()
    ///         .map(|n| n.value())
    ///         .collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn sort_by_id_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &T) -> K,
        K: Ord,
    {
        if self.has_children() {
            let (unsorted, sorted) = self.sort_handler(|nodes| {
                nodes.sort_by_key(|node| f(node.0.to_index(), node.1));
            });
            self.swap(unsorted, sorted);
        }
    }

    /// Applies a sorting function to the children of the current node and returns their IDs
    /// before and after sorting.
    ///
    /// This function takes a mutable closure `f` that sorts a vector of tuples,
    /// where each tuple consists of a `NodeId` and a reference to the node's value `&T`.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<NodeId>`: The original order of the children's `NodeId`s before sorting.
    /// - `Vec<NodeId>`: The order of the children's `NodeId`s after applying the sorting function.
    fn sort_handler<F>(&mut self, mut f: F) -> (Vec<NodeId>, Vec<NodeId>)
    where
        F: FnMut(&mut Vec<(NodeId, &T)>),
    {
        let children = unsafe { self.tree.get_unchecked(self.id()).children() };
        let (unsorted, mut nodes): (Vec<_>, Vec<_>) =
            children.map(|n| (n.id(), (n.id(), n.value()))).unzip();
        f(&mut nodes);
        let sorted = nodes.into_iter().map(|(id, _)| id).collect::<Vec<_>>();
        (unsorted, sorted)
    }

    /// Reorders the children of the current node to match the specified sorted order.
    ///
    /// This method takes two vectors of `NodeId`s: `unsorted`, which represents the original
    /// order of the node's children, and `sorted`, which represents the desired order after sorting.
    /// It swaps nodes in the tree such that their order in the tree matches the `sorted` vector.
    ///
    /// # Parameters
    ///
    /// - `unsorted`: A vector of `NodeId`s representing the original order of the node's children.
    /// - `sorted`: A vector of `NodeId`s representing the desired order of the node's children.
    ///
    /// # Safety
    ///
    /// This function uses unsafe code to access and modify the tree nodes. Ensure that the node
    /// indices are valid and that the tree structure remains consistent after the operation.
    fn swap(&mut self, unsorted: Vec<NodeId>, sorted: Vec<NodeId>) {
        let mut swap = |sorted_id: NodeId, unsorted_id: NodeId| {
            let mut node = unsafe { self.tree.get_unchecked_mut(unsorted_id) };
            node.insert_id_before(sorted_id);
        };

        let mut cache = None;
        let mut unsorted = unsorted.into_iter();
        for (index, &id) in sorted.iter().enumerate() {
            match cache {
                Some(cache_id) if cache_id != id => {
                    swap(id, cache_id);
                }
                Some(_) => cache = None,
                None => {
                    for unsorted_id in unsorted.by_ref() {
                        // Pass through the swapped elements.
                        if sorted
                            .iter()
                            .position(|&node| node == unsorted_id)
                            .is_some_and(|uindex| uindex < index)
                        {
                            continue;
                        }
                        if unsorted_id != id {
                            swap(id, unsorted_id);
                            cache = Some(unsorted_id);
                            break;
                        }
                    }
                }
            }
        }
    }
}
