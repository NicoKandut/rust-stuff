pub fn scheduleWithFixedDelay(f: fn() -> Void, delay: usize) {
  loop {
    std::thread::spawn(move || f());
    std::thread::sleep(delay);
  }
}
