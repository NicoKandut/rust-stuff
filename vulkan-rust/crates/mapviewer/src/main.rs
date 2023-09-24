use world::{gen::chunk::ChunkGenerator, ChunkId};

const MATERIAL_COLORS: [[u8; 3]; 10] = [
    [0, 0, 255],
    [10, 10, 255],
    [0, 255, 0],
    [0, 150, 0],
    [50, 100, 10],
    [255, 200, 100],
    [128, 255, 0],
    [255, 255, 255],
    [128, 255, 200],
    [0, 128, 64],
];

#[repr(u8)]
enum Biome {
    Ocean,
    FrozenOcean,
    Grassland,
    Forest,
    Jungle,
    Desert,
    Shrubland,
    Snowfield,
    Tundra,
    Taiga,
}

enum Land {
    Land,
    Ocean,
}

enum Temp {
    Hot,
    Regular,
    Cold,
}

enum Rainfall {
    Heavy,
    Regular,
    Sparse,
}

pub fn main() {
    println!("Starting generation");

    let mut bytes: Vec<u8> = Vec::new();

    let size: i32 = 512;
    let chunk_size = 1;
    for chunk_y in 0..size {
        let mut chunk_bytes: Vec<Vec<u8>> = Vec::new();

        for chunk_x in 0..size {
            chunk_bytes.push(Vec::new());

            let id = ChunkId::new(chunk_x, chunk_y, 0);
            let mut generator = ChunkGenerator::new();
            generator.generate_inplace(&id);

            for _chunk_y in 0..chunk_size {
                for _chunk_x in 0..chunk_size {
                    let land = if generator.wip_chunk_continentalness[1][1] > 0.0 {
                        Land::Land
                    } else {
                        Land::Ocean
                    };
                    let temp = match generator.wip_chunk_temperature[1][1] {
                        x if x > 0.01 => Temp::Hot,
                        x if x > -0.01 => Temp::Regular,
                        _ => Temp::Cold,
                    };

                    let rain = match generator.wip_chunk_rainfall[1][1] {
                        x if x > 0.01 => Rainfall::Heavy,
                        x if x > -0.01 => Rainfall::Regular,
                        _ => Rainfall::Sparse,
                    };

                    let biome: Biome = match (land, temp, rain) {
                        (Land::Land, Temp::Hot, Rainfall::Sparse) => Biome::Desert,
                        (Land::Land, Temp::Hot, Rainfall::Regular) => Biome::Shrubland,
                        (Land::Land, Temp::Hot, Rainfall::Heavy) => Biome::Jungle,
                        (Land::Land, Temp::Regular, Rainfall::Sparse) => Biome::Grassland,
                        (Land::Land, Temp::Regular, Rainfall::Regular) => Biome::Forest,
                        (Land::Land, Temp::Regular, Rainfall::Heavy) => Biome::Forest,
                        (Land::Land, Temp::Cold, Rainfall::Sparse) => Biome::Taiga,
                        (Land::Land, Temp::Cold, Rainfall::Regular) => Biome::Tundra,
                        (Land::Land, Temp::Cold, Rainfall::Heavy) => Biome::Snowfield,
                        (Land::Ocean, Temp::Cold, _) => Biome::FrozenOcean,
                        (Land::Ocean, _, _) => Biome::Ocean,
                    };

                    let mat = match biome {
                        Biome::Ocean => 0,
                        Biome::FrozenOcean => 1,
                        Biome::Grassland => 2,
                        Biome::Forest => 3,
                        Biome::Jungle => 4,
                        Biome::Desert => 5,
                        Biome::Shrubland => 6,
                        Biome::Snowfield => 7,
                        Biome::Tundra => 8,
                        Biome::Taiga => 9,
                    };

                    chunk_bytes[chunk_x as usize].extend(MATERIAL_COLORS[mat]);
                }
            }

            // println!("Chunk done");
        }

        for y in 0..chunk_size {
            for chunk_x in 0..size {
                let from = y * chunk_size * 3;
                let to = from + chunk_size * 3;
                let new_bytes = &chunk_bytes[chunk_x as usize][from..to];
                bytes.extend_from_slice(new_bytes);
            }
        }
    }

    println!("Saving");

    let image_size = (size as usize * chunk_size) as u32;
    image::save_buffer_with_format(
        "./map.png",
        &bytes,
        image_size,
        image_size,
        image::ColorType::Rgb8,
        image::ImageFormat::Png,
    )
    .expect("Saving image failed");

    println!("Done");
}
