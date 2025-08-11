use std::time::SystemTime;

fn main() {
	let elapsed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
	let value = elapsed.as_micros() % 1000;
	println!("{value}");
}
