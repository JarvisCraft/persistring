use {
    crate::{util::BytesSegment as Segment, PersistentString, VersionSwitchError},
    std::borrow::Cow,
};

type NodeAddress = usize;

#[derive(Debug)]
struct RopePersistentString {
    /// Buffer of the created string.
    buffer: String,
    /// **Arena**-like storage of used nodes.
    /// This never has to be cleaned up.
    nodes: Vec<Node>,
    /// Indices of root nodes corresponding to the versiosn.
    versions: Vec<NodeAddress>,
    /// Index of the current version
    current_version: usize,
}

impl RopePersistentString {
    fn version_node(&self, index: usize) -> &Node {
        &self.nodes[self.node_address(index)]
    }

    fn current_version_node(&self) -> &Node {
        &self.nodes[self.current_node_address()]
    }

    fn node_address(&self, index: usize) -> NodeAddress {
        self.versions[index]
    }

    fn current_node_address(&self) -> NodeAddress {
        self.node_address(self.current_version)
    }

    fn build_snapshot(&self, node: &NodeBody, result: &mut String) {
        match node {
            NodeBody::Leaf(segment) => result.push_str(segment.as_str(self.buffer.as_bytes())),
            NodeBody::Parent(left, right) => {
                self.build_snapshot(&self.nodes[*left].body, result);
                self.build_snapshot(&self.nodes[*right].body, result);
            }
        }
    }

    // note: `node` is taken by a (cloned) value in order to make borrowing checker happy
    fn pop_nonempty_recursively(&mut self, node: Node) -> (NodeAddress, char) {
        match node.body {
            NodeBody::Leaf(segment) => {
                let last_char = segment
                    .as_str(self.buffer.as_bytes())
                    .chars()
                    .last()
                    .expect("no empty segments should be present in the place other than root");

                let last_char_length = last_char.len_utf8();
                (
                    match segment.len() - last_char_length {
                        // eliminate this node
                        0 => 0,
                        new_length => {
                            let address = self.nodes.len();
                            self.nodes.push(Node {
                                length: new_length,
                                body: NodeBody::Leaf({
                                    let mut segment = segment.clone();
                                    segment.end -= last_char_length;
                                    segment
                                }),
                            });

                            address
                        }
                    },
                    last_char,
                )
            }
            NodeBody::Parent(left, right) => {
                debug_assert!(left != 0 && right != 0, "children cannot be empty");

                let (new_right, popped) = self.pop_nonempty_recursively(self.nodes[right].clone());
                (
                    match new_right {
                        // "pull" left node up if right became empty
                        0 => left,
                        // "replace" current node
                        new_right => {
                            let address = self.nodes.len();
                            self.nodes.push(Node {
                                length: node.length - popped.len_utf8(),
                                body: NodeBody::Parent(left, new_right),
                            });

                            address
                        }
                    },
                    popped,
                )
            }
        }
    }

    // note: `node` is taken by a (cloned) value in order to make borrowing checker happy
    fn insert_str_recursively(
        &mut self,
        node: Node,
        node_address: NodeAddress,
        insertion: Segment,
        index: usize,
    ) -> NodeAddress {
        match node.body {
            NodeBody::Leaf(segment) => {
                let inserted_node_address = self.nodes.len();
                self.nodes.push(Node::of_segment(insertion));

                let new_address;
                match index {
                    // insert as a left child
                    0 => {
                        new_address = self.nodes.len();
                        self.nodes.push(Node {
                            length: node.length + insertion.len(),
                            body: NodeBody::Parent(inserted_node_address, node_address),
                        })
                    }
                    // insert as a right child
                    index if index == segment.len() => {
                        new_address = self.nodes.len();
                        self.nodes.push(Node {
                            length: node.length + insertion.len(),
                            body: NodeBody::Parent(node_address, inserted_node_address),
                        })
                    }
                    // insert in the middle
                    index => {
                        let (left_segment, right_segment) = segment.split_at(index);

                        let left_address = self.nodes.len();
                        self.nodes.push(Node::of_segment(left_segment));

                        let right_address = self.nodes.len();
                        self.nodes.push(Node::of_segment(right_segment));

                        let left_pair_address = self.nodes.len();
                        self.nodes.push(Node {
                            length: left_segment.len() + insertion.len(),
                            body: NodeBody::Parent(left_address, inserted_node_address),
                        });

                        new_address = self.nodes.len();
                        self.nodes.push(Node {
                            length: segment.len() + insertion.len(),
                            body: NodeBody::Parent(left_pair_address, right_address),
                        })
                    }
                }
                new_address
            }
            NodeBody::Parent(left_address, right_address) => {
                let left_node = &self.nodes[left_address];
                let left_length = left_node.length;

                let new_address;
                if index <= left_length {
                    let new_left_address = self.insert_str_recursively(
                        left_node.clone(),
                        left_address,
                        insertion,
                        index,
                    );

                    new_address = self.nodes.len();
                    self.nodes.push(Node {
                        length: node.length + insertion.len(),
                        body: NodeBody::Parent(new_left_address, right_address),
                    });
                } else {
                    let new_right_address = self.insert_str_recursively(
                        self.nodes[right_address].clone(),
                        right_address,
                        insertion,
                        index - left_length,
                    );

                    new_address = self.nodes.len();
                    self.nodes.push(Node {
                        length: node.length + insertion.len(),
                        body: NodeBody::Parent(left_address, new_right_address),
                    });
                }

                new_address
            }
        }
    }
}

impl PersistentString for RopePersistentString {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            nodes: vec![Node {
                length: 0,
                body: NodeBody::Leaf(Segment::EMPTY),
            }],
            versions: vec![0],
            current_version: 0,
        }
    }

    fn version(&self) -> usize {
        self.current_version
    }

    fn latest_version(&self) -> usize {
        self.versions.len() - 1
    }

    fn try_switch_version(&mut self, version: usize) -> Result<(), VersionSwitchError> {
        if version < self.versions.len() {
            self.current_version = version;
            Ok(())
        } else {
            Err(VersionSwitchError::InvalidVersion(version))
        }
    }

    fn snapshot(&self) -> Cow<str> {
        let Node { length, body } = &self.nodes[self.versions[self.current_version]];

        match body {
            NodeBody::Leaf(segment) => Cow::Borrowed(segment.as_str(self.buffer.as_bytes())),
            NodeBody::Parent(_, _) => {
                let mut buffer = String::with_capacity(*length);
                self.build_snapshot(body, &mut buffer);
                Cow::Owned(buffer)
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.current_version_node().length == 0
    }

    fn len(&self) -> usize {
        self.current_version_node().length
    }

    fn pop(&mut self) -> Option<char> {
        // take current node but with the last node replaced with a "smaller one"
        // in order to do it, create a new path of right nodes
        // which will have right children recursively replaced with new ones

        let current_version = self.current_version_node();

        if current_version.length == 0 {
            return None;
        }

        let (new_root, popped) = self.pop_nonempty_recursively(self.current_version_node().clone());

        let new_version = self.versions.len();
        self.versions.push(new_root);
        self.current_version = new_version;

        Some(popped)
    }

    fn push(&mut self, character: char) {
        let new_version = self.versions.len();

        let character_length = character.len_utf8();

        let current_node_address = self.current_node_address();
        let right_node_index = self.nodes.len();

        {
            // push right node
            let suffix_begin = self.buffer.len();
            self.buffer.push(character);

            self.nodes.push(Node {
                length: character_length,
                body: NodeBody::Leaf(Segment::non_empty_of_length(suffix_begin, character_length)),
            });
        }

        let new_node_address = self.nodes.len();
        self.nodes.push(Node {
            length: self.nodes[current_node_address].length + character_length,
            body: NodeBody::Parent(current_node_address, right_node_index),
        });

        self.versions.push(new_node_address);

        self.current_version = new_version;
    }

    fn push_str(&mut self, suffix: &str) {
        let new_version = self.versions.len();

        let suffix_length = suffix.len();

        let current_node_address = self.current_node_address();

        let new_node_address;
        if suffix_length == 0 {
            // keep current string
            new_node_address = current_node_address;
        } else {
            let right_node_index = self.nodes.len();

            {
                // push right node
                let suffix_begin = self.buffer.len();
                self.buffer.push_str(suffix);

                self.nodes.push(Node {
                    length: suffix.len(),
                    body: NodeBody::Leaf(Segment::non_empty_of_length(suffix_begin, suffix_length)),
                });
            }

            if current_node_address == 0 {
                // no need to append anything to empty node if a new one can be the only node
                new_node_address = right_node_index;
            } else {
                new_node_address = self.nodes.len();
                self.nodes.push(Node {
                    length: self.nodes[current_node_address].length + suffix_length,
                    body: NodeBody::Parent(current_node_address, right_node_index),
                });
            }
        }

        self.versions.push(new_node_address);
        self.current_version = new_version;
    }

    fn repeat(&mut self, times: usize) {
        let new_version = self.versions.len();

        let current_node_index = self.current_node_address();
        if current_node_index == 0 {
            // node 0 is known to be empty thus there is
            // no need to increase the number of empty nodes
            self.versions.push(0);
        } else {
            match times {
                // the string should just become empty
                0 => self.versions.push(0),
                // the string is kept untouched
                1 => self.versions.push(current_node_index),
                // pair (a common scenario)
                2 => {
                    let length = self.nodes[current_node_index].length;

                    let node_pair_index = self.nodes.len();
                    self.nodes.push(Node {
                        length: length * 2,
                        body: NodeBody::Parent(current_node_index, current_node_index),
                    });
                    self.versions.push(node_pair_index);
                }
                times => {
                    let length = self.nodes[current_node_index].length;

                    let mut top_length = length;
                    let mut top_index = current_node_index;

                    for _ in 2..=times {
                        top_length += length;
                        let new_top_index = self.nodes.len();

                        self.nodes.push(Node {
                            length: top_length,
                            body: NodeBody::Parent(top_index, current_node_index),
                        });
                        top_index = new_top_index;
                    }
                    self.versions.push(top_index);
                    // TODO: balanced tree structure
                    /* // TODO: smart array pre-allocation
                    // [0] = 1, [1] = 2, [2] = 4, ...
                    let mut power_indices = Vec::new();

                    power_indices.push(current_node_index);

                    let mut pair_size = 1usize;
                    while {
                        pair_size *= 2;
                        pair_size <= times
                    } {}*/
                } // build a balanced tree using node reusage
            }
        }

        self.current_version = new_version;
    }

    fn remove(&mut self, index: usize) -> char {
        todo!()
    }

    fn retain(&mut self, filter: impl Fn(char) -> bool) {
        todo!()
    }

    fn insert(&mut self, index: usize, character: char) {
        self.insert_str(index, character.encode_utf8(&mut [0u8; 4]));
    }

    fn insert_str(&mut self, index: usize, insertion: &str) {
        let current_node_address = self.current_node_address();
        let node = &self.nodes[current_node_address];
        if index > node.length {
            panic!("index {} exceeds length {}", index, node.length);
        }

        let new_node_address;
        if insertion.is_empty() {
            new_node_address = current_node_address;
        } else {
            let insertion_begin = self.buffer.len();
            self.buffer.push_str(insertion);
            let segment = Segment::of_length(insertion_begin, insertion.len());

            if node.length == 0 {
                new_node_address = self.nodes.len();
                self.nodes.push(Node::of_segment(segment));
            } else {
                new_node_address =
                    self.insert_str_recursively(node.clone(), current_node_address, segment, index);
            }
        }
        let new_version = self.versions.len();
        self.versions.push(new_node_address);
        self.current_version = new_version;
    }
}

#[derive(Debug, Clone)]
struct Node {
    /// Length of the string represented ny this node or its children.
    /// Unlike [`Segment`]'s length, this one is UTF8-wise.
    length: usize,
    /// Body of this node.
    body: NodeBody,
}

impl Node {
    fn of_segment(segment: Segment) -> Self {
        Self {
            length: segment.len(),
            body: NodeBody::Leaf(segment),
        }
    }
}

#[derive(Debug, Clone)]
enum NodeBody {
    /// Leaf corresponding to some text.
    Leaf(Segment),
    /// Node which is parent of the other nodes.
    Parent(NodeAddress, NodeAddress),
}

#[cfg(test)]
mod tests {
    crate::tests::persistent_string_test_suite!(super::RopePersistentString);
}
