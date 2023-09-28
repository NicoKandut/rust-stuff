use std::collections::{HashMap, HashSet};
use world::{ChunkId, CHUNK_SIZE_F};

pub(crate) enum ChunkAction {
    Load(ChunkId),
    Unload(ChunkId),
}

pub(crate) struct ChunkTracker {
    load_distance: f32,
    unload_distance: f32,
    loaded: HashSet<ChunkId>,
    center: glm::Vec3,
}

impl ChunkTracker {
    pub(crate) fn new(load_distance: f32, unload_distance: f32) -> Self {
        Self {
            load_distance,
            unload_distance,
            loaded: Default::default(),
            center: Default::default(),
        }
    }

    pub(crate) fn set_distances(
        &mut self,
        load_distance: f32,
        unload_distance: f32,
    ) -> Vec<ChunkAction> {
        self.load_distance = load_distance;
        self.unload_distance = unload_distance;
        self.get_needed_actions()
    }

    pub(crate) fn set_center(&mut self, center: glm::Vec3) -> Vec<ChunkAction> {
        self.center = center;
        self.get_needed_actions()
    }

    fn get_needed_actions(&mut self) -> Vec<ChunkAction> {
        let mut actions = vec![];

        self.add_load_actions(&mut actions);
        self.add_unload_actions(&mut actions);

        actions
    }

    fn add_load_actions(&mut self, actions: &mut Vec<ChunkAction>) {
        let start = glm::IVec3::new(
            ((self.center.x - self.load_distance) / CHUNK_SIZE_F).floor() as i32,
            ((self.center.y - self.load_distance) / CHUNK_SIZE_F).floor() as i32,
            ((self.center.z - self.load_distance) / CHUNK_SIZE_F).floor() as i32,
        );

        let end = glm::IVec3::new(
            ((self.center.x + self.load_distance) / CHUNK_SIZE_F).ceil() as i32,
            ((self.center.y + self.load_distance) / CHUNK_SIZE_F).ceil() as i32,
            ((self.center.z + self.load_distance) / CHUNK_SIZE_F).ceil() as i32,
        );

        let size = (end - start).abs();

        let mut distances = HashMap::with_capacity((size.x * size.y * size.z) as usize);

        for z in start.z..end.z {
            for y in start.y..end.y {
                for x in start.x..end.x {
                    let candidate = ChunkId::new(x, y, z);

                    if self.loaded.contains(&candidate) {
                        continue;
                    }

                    let distance = glm::distance2(&self.center, &candidate.center());
                    if distance <= self.load_distance.powi(2) {
                        distances.insert(candidate, distance);
                    }
                }
            }
        }

        let mut ids_to_load = Vec::with_capacity(distances.len());
        ids_to_load.extend(distances.keys());
        ids_to_load.sort_unstable_by(|a, b| distances[a].total_cmp(&distances[b]));

        for id in ids_to_load {
            self.loaded.insert(id);
            actions.push(ChunkAction::Load(id))
        }
    }

    fn add_unload_actions(&mut self, actions: &mut Vec<ChunkAction>) {
        let ids_to_remove = self
            .loaded
            .iter()
            .filter(|chunk| {
                glm::distance2(&chunk.center(), &self.center) > self.unload_distance.powi(2)
            })
            .cloned()
            .collect::<Vec<_>>();

        for id in ids_to_remove {
            self.loaded.remove(&id);
            actions.push(ChunkAction::Unload(id))
        }
    }
}

#[cfg(test)]
mod tests {

    use test::Bencher;
    use world::{CHUNK_SIZE_F, HALF_CHUNK_F};

    use crate::chunk_stream::ChunkTracker;

    fn test_full_load(center: glm::Vec3, load_distance: f32, expected_chunks: usize) {
        let mut stream = ChunkTracker::new(load_distance, f32::MAX);
        let actions = stream.set_center(center);
        assert_eq!(actions.len(), expected_chunks);
        assert_eq!(stream.loaded.len(), expected_chunks);
    }

    #[test]
    fn loads_all_chunks_with_correct_distance() {
        // evens
        let center = glm::vec3(0.0, 0.0, 0.0);
        test_full_load(center, CHUNK_SIZE_F * 1.0, 8);
        test_full_load(center, CHUNK_SIZE_F * 2.0, 32);

        // odds
        let center = glm::vec3(32.0, 32.0, 32.0);
        test_full_load(center, CHUNK_SIZE_F * 0.50, 1);
        test_full_load(center, CHUNK_SIZE_F * 1.00, 7);
        test_full_load(center, CHUNK_SIZE_F * 1.50, 19);
        test_full_load(center, CHUNK_SIZE_F * 1.75, 27);
    }

    //   fn test_additive_load(center: &glm::Vec3, load_distance: f32, loaded_chunks:  expected_new_chunks: usize) {
    //     let mut stream = ChunkStream::new(load_distance, f32::MAX);
    //     let actions = stream.update(center);
    //     assert_eq!(actions.len(), expected_new_chunks);
    //     assert_eq!(stream.loaded.len(), expected_new_chunks);
    // }

    //   #[test]
    //   fn loads_additional_chunks() {
    //       let mut excluded = BTreeSet::new();
    //       excluded.insert(ChunkId::new(0, 0, 0));

    //       let center = WorldPosition::new(0, 0, 0);
    //       let ids = get_chunks_to_render(&center, CHUNK_SIZE_I * 2, &excluded);
    //       assert_eq!(ids.len(), 31);

    //       let center = WorldPosition::new(32, 32, 32);
    //       let ids = get_chunks_to_render(
    //           &center,
    //           CHUNK_SIZE_I + CHUNK_SIZE_I / 2 + CHUNK_SIZE_I / 4,
    //           &excluded,
    //       );
    //       assert_eq!(ids.len(), 26);
    //   }

    #[bench]
    fn first_load_6(b: &mut Bencher) {
        let center = glm::vec3(0.0, 0.0, 0.0);

        b.iter(|| {
            test::black_box({
                let mut stream = ChunkTracker::new(CHUNK_SIZE_F * 6.0, f32::MAX);
                stream.set_center(center);
            });
        });
    }

    #[bench]
    fn diagonal_move_full_chunk_6(b: &mut Bencher) {
        let mut stream = ChunkTracker::new(CHUNK_SIZE_F * 6.0, f32::MAX);
        let mut center = glm::vec3(0.0, 0.0, 0.0);

        b.iter(|| {
            center += glm::vec3(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F);
            test::black_box(stream.set_center(center));
        });
    }

    #[bench]
    fn straight_move_half_chunk_6(b: &mut Bencher) {
        let mut stream = ChunkTracker::new(CHUNK_SIZE_F * 6.0, f32::MAX);
        let mut center = glm::vec3(0.0, 0.0, 0.0);

        b.iter(|| {
            center += glm::vec3(HALF_CHUNK_F, 0.0, 0.0);
            test::black_box(stream.set_center(center));
        });
    }
}
