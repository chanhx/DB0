mod node;

use {
    self::node::{InternalNode, LeafNode, Node},
    core::ptr::addr_of_mut,
};

pub struct BTree<K, V> {
    root: Node<K, V>,

    max_keys: usize,
    mid_key_index: usize,
}

impl<K, V> BTree<K, V>
where
    K: Ord + Clone + Copy,
{
    pub fn new(max_keys: usize) -> Self {
        BTree {
            root: Node::Leaf(LeafNode::new(max_keys)),
            max_keys,
            mid_key_index: max_keys / 2,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut node = &mut self.root;

        // Optimize: with_capacity(level)
        let mut nodes = vec![];
        let mut splited_node;

        loop {
            match node {
                Node::Internal(internal) => {
                    let idx = internal.keys.binary_search(&key).unwrap_or_else(|i| i);
                    unsafe {
                        node = internal.children[idx].as_mut().unwrap();
                    }

                    nodes.push((internal, idx));
                }
                Node::Leaf(leaf) => {
                    match leaf.insert(key, value, self.max_keys, self.mid_key_index) {
                        Some(new) => {
                            splited_node = Node::Leaf(new);
                            break;
                        }
                        None => return,
                    }
                }
            }
        }

        let mut nodes_iter = nodes.into_iter().rev();
        loop {
            let key = match &splited_node {
                Node::Internal(internal) => internal.keys[0],
                Node::Leaf(leaf) => leaf.kvs[0].0,
            };

            match nodes_iter.next() {
                Some((node, idx)) => {
                    match node.insert(idx, key, splited_node, self.max_keys, self.mid_key_index) {
                        Some(internal) => splited_node = Node::Internal(internal),
                        None => return,
                    }
                }
                None => {
                    let mut keys = Vec::with_capacity(self.max_keys);
                    keys.push(key);

                    let mut children = Vec::with_capacity(self.max_keys);
                    // match (self.root, splited_node) {
                    //     (Node::Leaf(l1), Node::Leaf(l2)) => {
                    //         l2.next = l1.next;
                    //         l1.next = Some(addr_of_mut!(l2));
                    //     }
                    // }
                    children.push(addr_of_mut!(self.root));
                    children.push(addr_of_mut!(splited_node));

                    self.root = Node::Internal(InternalNode { keys, children });
                    return;
                }
            }
        }
    }

    pub fn search(&self, key: &K) -> Option<&V> {
        let mut node = &self.root;
        loop {
            match node {
                Node::Internal(internal) => {
                    let idx = internal.keys.binary_search(key).unwrap_or_else(|i| i);
                    // node = &internal.children[idx];
                    unsafe {
                        node = internal.children[idx].as_mut().unwrap();
                    }
                }
                Node::Leaf(leaf) => {
                    return leaf
                        .kvs
                        .binary_search_by_key(key, |(k, _)| *k)
                        .ok()
                        .map(|i| &leaf.kvs[i].1);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search() {
        let mut tree = BTree::new(5);

        for i in 0..=5 {
            tree.insert(i, i * 2);
        }
        for i in 0..=5 {
            assert_eq!(tree.search(&i), Some(&(i * 2)));
        }

        assert_eq!(tree.search(&16), None);
    }
}
