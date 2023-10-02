#![feature(variant_count)]

use std::env;
use clap::Command;
use gamedata::material::Material;
use std::mem::variant_count;
use xtaskops::ops::{clean_files};
use std::process::Command as Cmd;

fn main() {
    let cli = Command::new("xtask")
        .subcommand(Command::new("palette"))
        .subcommand(Command::new("clean"))
        .subcommand(Command::new("shader"));

    let matches = cli.get_matches();
    match matches.subcommand() {
        Some(("palette", _)) => generate_palette_file(),
        Some(("clean", _)) => clean_project(),
        Some(("shader", _)) => compile_shaders(),
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

fn clean_project() {
    println!("Cleaning...");
    clean_files("**/gfxrecon_capture_*").expect("Cleaning GPU captures must succeed");
    println!("Done");
}

fn compile_shaders() {
    let root = env::current_dir().unwrap().to_str().unwrap().to_owned();
    let shader_dir = root.clone() + "\\shader";

    let vulkan_dir = env::var("VULKAN_SDK").unwrap();
    let glsl_compile = vulkan_dir + "\\Bin\\glslc.exe";

    println!("Recompiling shaders...");

    let vert_err = Cmd::new(&glsl_compile)
        .current_dir(&shader_dir)
        .arg("shader.vert")
        .arg("-o")
        .arg("vert.spv")
        .output()
        .expect("Failed to compile vertex shader").stderr;

    if !vert_err.is_empty() {
        println!("Error in vertex shader\n{}", String::from_utf8(vert_err).unwrap());
    }

    let frag_err = Cmd::new(&glsl_compile)
        .current_dir(&shader_dir)
        .arg("shader.frag")
        .arg("-o")
        .arg("frag.spv")
        .output()
        .expect("Failed to compile vertex shader").stderr;

    if !frag_err.is_empty() {
        println!("Error in fragment shader\n{}", String::from_utf8(frag_err).unwrap());
    }

    println!("Done");
}
