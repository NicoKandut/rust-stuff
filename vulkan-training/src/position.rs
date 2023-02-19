pub type Pos3 = (f64, f64, f64);

pub trait Moveable {
  fn translate_by(&mut self, x: f64, y: f64, z: f64);
}
