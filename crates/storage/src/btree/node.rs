use {
    common::pub_fields_struct,
    core::ptr::{self, addr_of_mut},
    std::mem,
};

pub enum Node<K, V> {
    Internal(InternalNode<K, V>),
    Leaf(LeafNode<K, V>),
}

pub_fields_struct! {
    struct InternalNode<K, V> {
        keys: Vec<K>,
        children: Vec<*mut Node<K, V>>,
    }

    struct LeafNode<K, V> {
        kvs: Vec<(K, V)>,
        next: Option<*mut Self>,
    }
}

impl<K, V> LeafNode<K, V>
where
    K: Ord + Copy,
{
    pub fn new(max_keys: usize) -> Self {
        Self {
            kvs: Vec::with_capacity(max_keys + 1),
            next: None,
        }
    }

    pub fn insert(
        &mut self,
        key: K,
        value: V,
        max_keys: usize,
        mid_key_index: usize,
    ) -> Option<Self> {
        match self.kvs.binary_search_by_key(&key, |&(k, _)| k) {
            Ok(i) => {
                self.kvs[i].1 = value;
                None
            }
            Err(i) => {
                self.kvs.insert(i, (key, value));

                if self.kvs.len() <= max_keys {
                    return None;
                }

                let mut right = Vec::with_capacity(max_keys + 1);

                // Unsafely `set_len` and copy items to `right`.
                unsafe {
                    right.set_len(self.kvs.len() - mid_key_index);
                    self.kvs.set_len(mid_key_index);

                    ptr::copy_nonoverlapping(
                        self.kvs.as_ptr().add(mid_key_index),
                        right.as_mut_ptr(),
                        right.len(),
                    );
                }

                let mut right = LeafNode {
                    kvs: right,
                    next: None,
                };
                mem::swap(&mut right.next, &mut self.next);
                self.next = Some(addr_of_mut!(right));

                Some(right)
            }
        }
    }
}

impl<K, V> InternalNode<K, V>
where
    K: Ord,
{
    pub fn insert(
        &mut self,
        index: usize,
        key: K,
        mut child: Node<K, V>,
        max_keys: usize,
        mid_key_index: usize,
    ) -> Option<Self> {
        self.keys.insert(index, key);
        self.children.insert(index, addr_of_mut!(child));

        if !self.keys.len() > max_keys {
            return None;
        }

        let mut right = Vec::with_capacity(max_keys + 1);
        let mut children = Vec::with_capacity(max_keys + 2);

        // Unsafely `set_len` and copy items to `right`.
        unsafe {
            right.set_len(self.keys.len() - mid_key_index);
            self.keys.set_len(mid_key_index);

            children.set_len(self.keys.len() - mid_key_index);
            self.children.set_len(mid_key_index);

            ptr::copy_nonoverlapping(
                self.keys.as_ptr().add(mid_key_index),
                right.as_mut_ptr(),
                right.len(),
            );

            ptr::copy_nonoverlapping(
                self.children.as_ptr().add(mid_key_index),
                children.as_mut_ptr(),
                children.len(),
            );
        }

        let right = InternalNode {
            keys: right,
            children,
        };

        Some(right)
    }
}
