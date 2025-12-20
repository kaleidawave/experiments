fn main() {
    let permutations = crate::utilities::Permutations::new(vec![vec![1, 2, 3], vec![10, 20, 30], vec![100, 200]]);
    let mut i = 0;
    for permutation in permutations {
        let prefix = &"     "[..i];
        println!("{prefix}{permutation:?}");
        i = (i + 1) % 3;
    }
}
