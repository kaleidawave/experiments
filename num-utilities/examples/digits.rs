fn main() {
    let x: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    dbg!(num_utilities::format::to_english(x));
    if x < 2300 {
        dbg!(num_utilities::format::to_roman_numeral(x));
    }
}
