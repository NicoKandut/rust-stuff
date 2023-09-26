use engine::Engine;

fn main() {
    println!("Starting game");
    Engine::create().run().unwrap();
    println!("Exiting game");
}
