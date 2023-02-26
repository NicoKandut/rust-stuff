use super::graphics::Vertex;
use crate::AppData;
use anyhow::Result;
use nalgebra_glm as glm;
use std::{collections::HashMap, fs::File, io::BufReader};

pub fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = BufReader::new(File::open(
        "crates/engine/src/resources/models/viking_room.obj",
    )?);

    let (models, _) = tobj::load_obj_buf(&mut reader, true, |_| {
        Ok((vec![tobj::Material::empty()], HashMap::new()))
    })?;

    let mut unique_vertices = HashMap::new();

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;

            let vertex = Vertex::new(
                glm::vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                glm::vec3(1.0, 1.0, 1.0),
                glm::vec3(1.0, 1.0, 1.0), //TODO: take normal from mesh
            );

            if let Some(index) = unique_vertices.get(&vertex) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertices.push(vertex);
                data.indices.push(index as u32);
            }
        }
    }

    Ok(())
}
