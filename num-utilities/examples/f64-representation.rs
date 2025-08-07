fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let arg = args.next().unwrap();
    let arg: f64 = match arg.as_str() {
        "+∞" | "∞" | "inf" | "infinity" => f64::INFINITY,
        "-∞" | "-inf" | "-infinity" => f64::NEG_INFINITY,
        "NaN" => f64::NAN,
        "max" => f64::MAX,
        "min" => f64::MIN,
        "minpos" => f64::MIN_POSITIVE,
        "epsilon" | "ε" => f64::EPSILON,
        arg => arg.parse().unwrap(),
    };

    let arg = arg.to_bits();
    eprintln!("{arg:#066b}");

    let empty = "";
    println!("|sign|exponent{empty:3}|fraction{empty:44}|");
    let sign = (arg >> 63) as u64;
    let exponent = 0b0111_1111_1111 & (arg >> 52) as u64;
    let fraction = (2u64.pow(52) - 1u64) & arg;
    println!("|{sign:<4b}|{exponent:011b}|{fraction:052b}|");

    Ok(())
}
