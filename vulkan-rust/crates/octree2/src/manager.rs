use crate::node::{self, Node};

#[derive(Default, Debug)]
pub struct NodeManager {
    nodes: Vec<NodeRc>,
}

#[derive(Debug)]
struct NodeRc {
    node: Node,
    use_count: usize,
}

pub struct NodeId(usize);

impl NodeManager {
    pub fn new() -> Self {
        Self {
            nodes: Vec::default(),
        }
    }

    pub fn index_of(&self, node: &Node) -> Option<usize> {
        let mut index = 0;

        for current_node in &self.nodes {
            if *node == current_node.node {
                return Some(index);
            }

            index += 1;
        }

        None
    }

    /**
     * Adds a node to the node manager. Returns true if the node was added and false if it was already present.
     */
    pub fn add(&mut self, node: Node) -> usize {
        if let Some(index) = self.index_of(&node) {
            self.nodes[index].use_count += 1;
            index
        } else {
            self.nodes.push(NodeRc { node, use_count: 1 });
            self.nodes.len() - 1
        }
    }

    /**
     * Attempts to remve a node from the node manager. Returns true if it was removed and false otherwise.
     */
    pub fn remove(&mut self, node: &Node) -> bool {
        if let Some(index) = self.index_of(&node) {
            self.nodes[index].use_count -= 1;
            true
        } else {
            false
        }
    }

    /**
     * Gets a reference to a node
     */
    pub fn get(&self, index: &usize) -> Option<&Node> {
        if let Some(node_rc) = self.nodes.get(*index) {
            Some(&node_rc.node)
        } else {
            None
        }
    }

    /**
     * Sets the child of a node
     */
    pub fn set_child_of(&mut self, parent_id: &usize, child_index: &usize, child_id: &usize) {
        if let Some(node_rc) = self.nodes.get_mut(*parent_id) {
            if node_rc.use_count == 1 {
                node_rc.node.set_child(child_index, child_id);
            } else {
                node_rc.use_count -= 1;

                let mut node = node_rc.node.clone();
                node.set_child(child_index, child_id);

                self.add(node);
            }
        } else {
            panic!("Attempted to set child of non-existent parent");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use rand::{rngs::ThreadRng, Rng, RngCore};
    use test::Bencher;

    use crate::{node::Node, NodeManager};

    #[test]
    fn add_same_node() {
        let node1 = Node::new([1, 1, 1, 1, 0, 0, 0, 0]);
        let node2 = Node::new([1, 1, 1, 1, 0, 0, 0, 0]);

        let mut manager = NodeManager::default();

        assert!(0 == manager.add(node1));
        assert!(0 == manager.add(node2));
    }

    #[test]
    fn add_different_nodes() {
        let node1 = Node::new([1, 1, 1, 1, 0, 0, 0, 0]);
        let node2 = Node::new([0, 0, 0, 0, 1, 1, 1, 1]);

        let mut manager = NodeManager::default();

        assert!(0 == manager.add(node1));
        assert!(1 == manager.add(node2));
    }

    #[bench]
    fn add_chunk_uniform(b: &mut Bencher) {
        let mut manager = NodeManager::new();

        b.iter(|| {
            test::black_box(for z in 0..32 {
                for y in 0..32 {
                    for x in 0..32 {
                        manager.add(Node::new([0, 0, 0, 0, 0, 0, 0, 0]));
                    }
                }
            });
        });
    }

    #[bench]
    fn add_chunk_checkered(b: &mut Bencher) {
        let mut manager = NodeManager::new();

        b.iter(|| {
            test::black_box(for z in 0..32 {
                for y in 0..32 {
                    for x in 0..32 {
                        manager.add(Node::new([
                            (z + y + x) % 2,
                            (z + y + x + 1) % 2,
                            (z + y + x) % 2,
                            (z + y + x + 1) % 2,
                            (z + y + x) % 2,
                            (z + y + x + 1) % 2,
                            (z + y + x) % 2,
                            (z + y + x + 1) % 2,
                        ]));
                    }
                }
            });
        });
    }

    #[bench]
    fn add_chunk_pseudo_random(b: &mut Bencher) {
        let mut manager = NodeManager::new();

        b.iter(|| {
            test::black_box(for z in 0..32 {
                for y in 0..32 {
                    for x in 0..32 {
                        manager.add(Node::new([
                            (z * 3 + y + x) % 2,
                            (z * 2 + y + x + 1) % 2,
                            (z * 17 + y + x + 12) % 2,
                            (z + y * 3 + x + 1) % 2,
                            (z + y * 32 + x) % 2,
                            (z + y * 7 + x + 1) % 2,
                            (z + y + 123 * 5 + x * 123) % 2,
                            (z + y + x * 4 + 1) % 2,
                        ]));
                    }
                }
            });
        });
    }

    #[test]
    fn size_of_random_chunk_3_materials() {
        let chunk_size = 64;
        let materials = 3;

        let mut manager = NodeManager::new();
        let mut random = rand::thread_rng();

        for z in 0..(chunk_size / 2) {
            for y in 0..(chunk_size / 2) {
                for x in 0..(chunk_size / 2) {
                    manager.add(Node::new([
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                        random.gen_range(0..materials),
                    ]));
                }
            }
        }

        // println!("{:#?}", manager);
        println!(
            "Raw: {} blocks ({} byte)",
            chunk_size * chunk_size * chunk_size,
            chunk_size * chunk_size * chunk_size * size_of::<usize>()
        );
        println!(
            "NodeManager: {} nodes ({} byte)",
            manager.nodes.len(),
            manager.nodes.len() * size_of::<Node>()
        );
    }
}