// use std::collections::{BTreeSet, HashMap};

// use world::{ChunkId, WorldPosition, CHUNK_SIZE_I};

// pub fn get_chunks_to_render(
//     center: &WorldPosition,
//     render_distance: i32,
//     exclude: &BTreeSet<ChunkId>,
// ) -> Vec<ChunkId> {
//     let render_distance_squared = render_distance.pow(2);

//     let start = ChunkId::new(
//         (center.x - render_distance) / CHUNK_SIZE_I - 1,
//         (center.y - render_distance) / CHUNK_SIZE_I - 1,
//         (center.z - render_distance) / CHUNK_SIZE_I - 1,
//     );

//     let end = ChunkId::new(
//         (center.x + render_distance) / CHUNK_SIZE_I + 1,
//         (center.y + render_distance) / CHUNK_SIZE_I + 1,
//         (center.z + render_distance) / CHUNK_SIZE_I + 1,
//     );

//     let max_chunk_count =
//         ((end.x - start.x) * (end.y - start.y) * (end.z - start.z)).abs() as usize;
//     let mut distances = HashMap::with_capacity(max_chunk_count);

//     for z in start.z..end.z {
//         for y in start.y..end.y {
//             for x in start.x..end.x {
//                 let candidate = ChunkId::new(x, y, z);
//                 if exclude.contains(&candidate) {
//                     continue;
//                 }

//                 let distance = WorldPosition::distance_squared(center, &candidate.center());
//                 if distance <= render_distance_squared {
//                     distances.insert(candidate, distance);
//                 }
//             }
//         }
//     }

//     let mut result = Vec::with_capacity(distances.len());
//     result.extend(distances.keys());
//     result.sort_by(|a, b| distances[a].cmp(&distances[b]));

//     result
// }

// #[derive(Debug)]
// pub(crate) struct ChunkLoadingAction {
//     pub(crate) add: Vec<ChunkId>,
//     pub(crate) remove: Vec<ChunkId>,
// }

// pub(crate) fn calculate_chunk_diff(
//     loaded_chunks: &BTreeSet<ChunkId>,
//     center: &WorldPosition,
//     render_distance: i32,
//     unrender_distance: i32,
// ) -> ChunkLoadingAction {
//     let remove = loaded_chunks
//         .iter()
//         .filter(|chunk| {
//             WorldPosition::distance_squared(center, &chunk.center()) > unrender_distance.pow(2)
//         })
//         .cloned()
//         .collect::<Vec<_>>();
//     let add = get_chunks_to_render(center, render_distance, loaded_chunks);

//     if !add.is_empty() || !remove.is_empty() {
//         println!(
//             "Loading chunks:\n   adding{:?}, removing: {:?}",
//             add.len(),
//             remove.len()
//         )
//     }

//     ChunkLoadingAction { add, remove }
// }

// #[cfg(test)]
// mod tests {
//     use std::collections::BTreeSet;

//     use test::Bencher;
//     use world::{ChunkId, WorldPosition, CHUNK_SIZE_I};

//     use super::get_chunks_to_render;

//     #[test]
//     fn includes_all_chunks() {
//         let empty = BTreeSet::default();
//         // evens
//         let center = WorldPosition::new(0, 0, 0);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I, &empty);
//         assert_eq!(ids.len(), 8);

//         let center = WorldPosition::new(0, 0, 0);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I * 2, &empty);
//         assert_eq!(ids.len(), 32);

//         // odds
//         let center = WorldPosition::new(32, 32, 32);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I / 2, &empty);
//         assert_eq!(ids.len(), 1);

//         let center = WorldPosition::new(32, 32, 32);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I, &empty);
//         assert_eq!(ids.len(), 7);

//         let center = WorldPosition::new(32, 32, 32);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I + CHUNK_SIZE_I / 2, &empty);
//         assert_eq!(ids.len(), 19);

//         let center = WorldPosition::new(32, 32, 32);
//         let ids = get_chunks_to_render(
//             &center,
//             CHUNK_SIZE_I + CHUNK_SIZE_I / 2 + CHUNK_SIZE_I / 4,
//             &empty,
//         );
//         assert_eq!(ids.len(), 27);
//     }

//     #[test]
//     fn with_excluded() {
//         let mut excluded = BTreeSet::new();
//         excluded.insert(ChunkId::new(0, 0, 0));

//         let center = WorldPosition::new(0, 0, 0);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I * 2, &excluded);
//         assert_eq!(ids.len(), 31);

//         let center = WorldPosition::new(32, 32, 32);
//         let ids = get_chunks_to_render(
//             &center,
//             CHUNK_SIZE_I + CHUNK_SIZE_I / 2 + CHUNK_SIZE_I / 4,
//             &excluded,
//         );
//         assert_eq!(ids.len(), 26);
//     }

//     #[bench]
//     fn first_load_6(b: &mut Bencher) {
//         let empty = BTreeSet::default();
//         let center = WorldPosition::new(0, 0, 0);

//         b.iter(|| {
//             test::black_box(get_chunks_to_render(&center, CHUNK_SIZE_I * 6, &empty));
//         });
//     }

//     #[bench]
//     fn diagonal_move_full_chunk_6(b: &mut Bencher) {
//         let empty = BTreeSet::default();
//         let center = WorldPosition::new(0, 0, 0);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I * 6, &empty);
//         let exclude = BTreeSet::from_iter(ids.into_iter());

//         let center = WorldPosition::new(CHUNK_SIZE_I, CHUNK_SIZE_I, CHUNK_SIZE_I);
//         b.iter(|| {
//             test::black_box(get_chunks_to_render(&center, CHUNK_SIZE_I * 6, &exclude));
//         });
//     }

//     #[bench]
//     fn straight_move_half_chunk_6(b: &mut Bencher) {
//         let empty = BTreeSet::default();
//         let center = WorldPosition::new(0, 0, 0);
//         let ids = get_chunks_to_render(&center, CHUNK_SIZE_I * 6, &empty);
//         let exclude = BTreeSet::from_iter(ids.into_iter());

//         let center = WorldPosition::new(CHUNK_SIZE_I / 2, 0, 0);
//         b.iter(|| {
//             test::black_box(get_chunks_to_render(&center, CHUNK_SIZE_I * 6, &exclude));
//         });
//     }
// }
