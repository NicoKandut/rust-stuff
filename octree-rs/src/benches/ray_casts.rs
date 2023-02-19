#[cfg(test)]
mod tests {

    use test::{black_box, Bencher};

    use crate::octree::{intersect, Node, Ray};

    #[bench]
    fn bench_100_rays_from_center(b: &mut Bencher) {
        let center = [0., 0., 0.];
        let node = Node::new(center, 1.);

        b.iter(|| {
            for _ in 1..100 {
                let ray = Ray::new(node.center, [0.1, 0.1, 0.1]);
                black_box(intersect(&node, &ray));
            }
        });
    }
}
