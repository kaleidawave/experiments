use spectra_lib::run_tests;

fn main() {
    let mut path = std::env::current_dir().unwrap();
    path.push("specification.md");
    run_tests(&path);
}
