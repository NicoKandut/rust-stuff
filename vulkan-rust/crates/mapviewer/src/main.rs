// #![feature(variant_count)]

use png::{BitDepth::Eight, ColorType::RGBA};
use rand::{thread_rng, Rng};
use std::{fs::File, io};

const CLOSED: [u8; SAMPLES] = [255, 0, 0, 255];
const OPEN: [u8; SAMPLES] = [0, 255, 0, 255];

const TILE_SIZE: usize = 8;
const TILE_SIZE_WITH_BORDER: usize = TILE_SIZE + 2;

const SAMPLES: usize = 4;

#[derive(Clone, Copy)]
struct Tile {
    pixels: [u8; TILE_SIZE * TILE_SIZE * 4],
    open: [bool; 4],
}

#[inline]
const fn index(x: usize, y: usize, w: usize, s: usize) -> usize {
    s * (y * w + x)
}

fn read_tileset(path: &str) -> Result<Vec<Tile>, io::Error> {
    let (info, pixels) = resources::read_image(path)?;

    let px_size = info.color_type.samples() * info.bit_depth as usize / 8;

    let width = info.width as usize;
    let nr_rows = info.height as usize / TILE_SIZE_WITH_BORDER;
    let nr_cols = info.width as usize / TILE_SIZE_WITH_BORDER;

    let mut tileset = Vec::with_capacity(16);

    for tile_y in 0..nr_rows {
        for tile_x in 0..nr_cols {
            let mut tile_pixels = [0_u8; TILE_SIZE * TILE_SIZE * 4];

            for row in 0..TILE_SIZE {
                let src_start = index(
                    tile_x * TILE_SIZE_WITH_BORDER + 1,
                    tile_y * TILE_SIZE_WITH_BORDER + 1 + row,
                    width,
                    px_size,
                );
                let src_end = index(
                    tile_x * TILE_SIZE_WITH_BORDER + 1 + TILE_SIZE,
                    tile_y * TILE_SIZE_WITH_BORDER + 1 + row,
                    width,
                    px_size,
                );
                let dst_start = index(0, row, TILE_SIZE, px_size);
                let dst_end = index(TILE_SIZE, row, TILE_SIZE, px_size);
                tile_pixels[dst_start..dst_end].copy_from_slice(&pixels[src_start..src_end]);
            }

            let indices = [
                index(
                    tile_x * TILE_SIZE_WITH_BORDER + 1,
                    tile_y * TILE_SIZE_WITH_BORDER,
                    width,
                    px_size,
                ),
                index(
                    tile_x * TILE_SIZE_WITH_BORDER + 1 + TILE_SIZE,
                    tile_y * TILE_SIZE_WITH_BORDER + 1,
                    width,
                    px_size,
                ),
                index(
                    tile_x * TILE_SIZE_WITH_BORDER + 1,
                    tile_y * TILE_SIZE_WITH_BORDER + 1 + TILE_SIZE,
                    width,
                    px_size,
                ),
                index(
                    tile_x * TILE_SIZE_WITH_BORDER,
                    tile_y * TILE_SIZE_WITH_BORDER + 1,
                    width,
                    px_size,
                ),
            ];

            let open = [
                OPEN == pixels[indices[0]..(indices[0] + px_size)],
                OPEN == pixels[indices[1]..(indices[1] + px_size)],
                OPEN == pixels[indices[2]..(indices[2] + px_size)],
                OPEN == pixels[indices[3]..(indices[3] + px_size)],
            ];

            tileset.push(Tile {
                pixels: tile_pixels,
                open,
            });
        }
    }

    Ok(tileset)
}

fn save_tileset(tileset: &[Tile], path: &str) -> Result<(), io::Error> {
    let px_size = RGBA.samples() * Eight as usize / 8;

    let mut pixels =
        vec![255_u8; TILE_SIZE_WITH_BORDER * TILE_SIZE_WITH_BORDER * tileset.len() * px_size];

    let mut tile_y = 0;
    for tile in tileset {
        let start = index(
            1,
            tile_y * TILE_SIZE_WITH_BORDER,
            TILE_SIZE_WITH_BORDER,
            px_size,
        );
        let end = index(
            1 + TILE_SIZE,
            tile_y * TILE_SIZE_WITH_BORDER,
            TILE_SIZE_WITH_BORDER,
            px_size,
        );
        let color = if tile.open[0] { OPEN } else { CLOSED };
        pixels[start..end].copy_from_slice(&color.repeat(TILE_SIZE));

        let left_color = if tile.open[3] { OPEN } else { CLOSED };
        let right_color = if tile.open[1] { OPEN } else { CLOSED };
        for row in 0..TILE_SIZE {
            let src_start = index(0, row, TILE_SIZE, px_size);
            let src_end = index(TILE_SIZE, row, TILE_SIZE, px_size);
            let dst_start = index(
                1,
                tile_y * TILE_SIZE_WITH_BORDER + 1 + row,
                TILE_SIZE_WITH_BORDER,
                px_size,
            );
            let dst_end = index(
                1 + TILE_SIZE,
                tile_y * TILE_SIZE_WITH_BORDER + 1 + row,
                TILE_SIZE_WITH_BORDER,
                px_size,
            );
            pixels[(dst_start - SAMPLES)..dst_start].copy_from_slice(&left_color);
            pixels[dst_start..dst_end].copy_from_slice(&tile.pixels[src_start..src_end]);
            pixels[dst_end..(dst_end + SAMPLES)].copy_from_slice(&right_color);
        }

        let start = index(
            1,
            tile_y * TILE_SIZE_WITH_BORDER + TILE_SIZE + 1,
            TILE_SIZE_WITH_BORDER,
            px_size,
        );
        let end = index(
            1 + TILE_SIZE,
            tile_y * TILE_SIZE_WITH_BORDER + TILE_SIZE + 1,
            TILE_SIZE_WITH_BORDER,
            px_size,
        );
        let color = if tile.open[2] { OPEN } else { CLOSED };
        pixels[start..end].copy_from_slice(&color.repeat(TILE_SIZE));

        tile_y += 1;
    }

    resources::write_image(
        path,
        &pixels,
        TILE_SIZE_WITH_BORDER,
        tileset.len() * TILE_SIZE_WITH_BORDER,
    )?;

    Ok(())
}

fn generate_map(tileset: &[Tile], width: usize, height: usize) -> Vec<Tile> {
    let mut map: Vec<Tile> = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let matches = tileset
                .iter()
                .filter(|tile| {
                    if y > 0 {
                        tile.open[0] == map[(y - 1) * width + x].open[2]
                    } else {
                        true
                    }
                })
                .filter(|tile| {
                    if x > 0 {
                        tile.open[3] == map[y * width + (x - 1)].open[1]
                    } else {
                        true
                    }
                })
                .collect::<Vec<_>>();

            let tile = matches[thread_rng().gen_range(0..matches.len())].clone();
            map.push(tile);
        }
    }

    map
}

pub fn main() {
    println!("Reading tileset");
    let tileset = read_tileset("assets/tileset.png").unwrap();

    println!("Saving copy");
    save_tileset(&tileset, "assets/tileset_copy.png").unwrap();

    println!("Generating map");
    let map = generate_map(&tileset, 10, 10);

    println!("Saving output");
    save_map(map, "assets/map.png", 10, 10).unwrap();

    println!("Done");
}

fn save_map(map: Vec<Tile>, path: &str, width: usize, height: usize) -> Result<(), io::Error> {
    let px_size = RGBA.samples() * Eight as usize / 8;
    let mut pixels = vec![255_u8; width * TILE_SIZE * height * TILE_SIZE * px_size];

    for tile_y in 0..height {
        for tile_x in 0..width {
            let tile = map[tile_y * width + tile_x];

            for row in 0..TILE_SIZE {
                let src_start = index(0, row, TILE_SIZE, px_size);
                let src_end = index(TILE_SIZE, row, TILE_SIZE, px_size);
                let dst_start = index(
                    tile_x * TILE_SIZE,
                    tile_y * TILE_SIZE + row,
                    width * TILE_SIZE,
                    px_size,
                );
                let dst_end = index(
                    tile_x * TILE_SIZE + TILE_SIZE,
                    tile_y * TILE_SIZE + row,
                    width * TILE_SIZE,
                    px_size,
                );

                pixels[dst_start..dst_end].copy_from_slice(&tile.pixels[src_start..src_end]);
            }
        }
    }

    resources::write_image(path, &pixels, width * TILE_SIZE, height * TILE_SIZE)?;

    Ok(())
}
