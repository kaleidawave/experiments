use std::env::args;
use std::hint::black_box;

fn get_iterations() -> usize {
    let first: String = args().nth(1).unwrap();
    first.parse().unwrap()
}

fn main() {
    eprintln!("running!!!");
    let iterations = get_iterations();
    run(iterations);
}

fn run(n: usize) {
    for _ in 0..n {
        black_box(noop);
    }
}

pub fn noop() {}
