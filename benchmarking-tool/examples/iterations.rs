use std::env::args;
use std::hint::black_box;

fn get_iterations() -> usize {
    let first: String = args().nth(1).unwrap();
    first.parse().unwrap()
}

fn main() {
    run(get_iterations());
}

fn run(n: usize) {
    for _ in 0..n {
        black_box(noop);
    }
}

pub fn noop() {}
