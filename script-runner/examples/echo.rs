fn main() {
	println!("{args:?}", args=std::env::args().skip(1).collect::<Vec<_>>());
}