use crate::position::Position;

#[derive(Default, PartialEq, Eq, Hash, Clone, Debug)]
pub struct Node([usize; 8]);

impl Node {
    pub fn new(children: [usize; 8]) -> Self {
        Self(children)
    }

    pub fn set_child(&mut self, index: &usize, child: &usize) {
        self.0[*index] = *child;
    }

    pub fn get_child_id(&self, index: usize) -> usize {
        return self.0[index];
    }
}
