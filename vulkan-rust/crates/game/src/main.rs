use anyhow::Result;
use engine::Engine;

fn main() -> Result<()> {
    let mut engine = Engine::create();
    engine.run()
}
