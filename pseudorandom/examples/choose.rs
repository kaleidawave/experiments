use std::time::SystemTime;

fn main() {
    let elapsed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let items: Vec<String> = std::env::args().skip(1).collect();

    let item: &str = &items[elapsed.as_micros() as usize % items.len()];

    println!("{item}");
}
