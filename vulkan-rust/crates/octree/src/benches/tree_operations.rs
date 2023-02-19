#[cfg(test)]
mod tests {
    use test::{black_box, Bencher};

    use crate::octree::{create_children, Node, Octree};

    #[bench]
    fn bench_splits(b: &mut Bencher) {
        let mut tree = Octree::new(1.0);

        b.iter(|| {
            let max = tree.len();
            for _ in 1..10 {
                black_box(tree.split_node(max - 1).expect("split failed"));
            }
        });
    }

    #[bench]
    fn bench_create_node(b: &mut Bencher) {
        b.iter(|| {
            for _ in 1..10 {
                black_box(Node::new([0., 0., 0.], 1.));
            }
        });
    }

    #[bench]
    fn bench_create_children(b: &mut Bencher) {
        let node = Node::new([0., 0., 0.], 1.);
        b.iter(|| {
            for _ in 1..10 {
                black_box(create_children(&node, 1));
            }
        });
    }
}
