#![feature(variant_count)]

use clap::Command;
use gamedata::material::Material;
use std::mem::variant_count;

fn main() {
    let cli = Command::new("xtask").subcommand(Command::new("palette"));

    let matches = cli.get_matches();
    match matches.subcommand() {
        Some(("palette", _)) => generate_palette_file(),
        _ => {}
    }
}

fn generate_palette_file() {
    println!("Generating palette...");
    let mut pixels = Vec::with_capacity(variant_count::<Material>());

    for material in Material::ALL {
        pixels.extend(material.color_bytes());
    }

    assert_eq!(pixels.len(), variant_count::<Material>() * 4);

    resources::write_image(
        "assets/palette.png",
        &pixels,
        variant_count::<Material>(),
        1,
    )
    .expect("Failed to generate palette");

    println!("Done");
}
