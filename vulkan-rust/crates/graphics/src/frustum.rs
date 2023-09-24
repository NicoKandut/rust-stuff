use crate::aabb::AABB;

pub struct Frustum {
    planes: [glm::Vec4; 4],
    #[allow(unused)]
    points: [glm::Vec3; 8],
}

impl Frustum {
    pub fn from_mat4(mat: &glm::Mat4) -> Self {
        let mat = mat.transpose();
        let planes = [
            mat.column(3) + mat.column(0), // left
            mat.column(3) - mat.column(0), // right
            mat.column(3) + mat.column(1), // bottom
            mat.column(3) - mat.column(1), // top
                                           // vp.column(2) + vp.column(3), // near
                                           // vp.column(2) - vp.column(3), // far
        ];

        Self {
            planes,
            points: Default::default(),
        }
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let AABB { min, max } = aabb;

        // check box outside/inside of frustum
        for plane in &self.planes {
            let out = 0
                + (glm::dot(plane, &glm::vec4(min.x, min.y, min.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(max.x, min.y, min.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(min.x, max.y, min.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(max.x, max.y, min.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(min.x, min.y, max.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(max.x, min.y, max.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(min.x, max.y, max.z, 1.)) < 0.) as u16
                + (glm::dot(plane, &glm::vec4(max.x, max.y, max.z, 1.)) < 0.) as u16;

            if out == 8 {
                return false;
            };
        }

        // check frustum outside/inside box
        // let mut out;
        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].x > max.x) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].x < min.x) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].y > max.y) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].y < min.y) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].z > max.z) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        // out = 0;
        // for i in 0..8 {
        //     out += (self.points[i].z < min.z) as u16
        // }
        // if out == 8 {
        //     return false;
        // };

        return true;
    }
}
