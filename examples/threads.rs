fn main() {
    let mut i = 0;
    loop {
        println!("thread {}", i);
        i += 1;
        spawn_thread();
    }
}

fn spawn_thread() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(1000));
    });
}
