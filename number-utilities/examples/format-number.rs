use number_utilities::format;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let value = args.next();
    if let Some("--help") | None = value.as_deref() {
        eprintln!(
            "usage: `{BIN_NAME} *number* (decinary)` or `{BIN_NAME} *number* --roman | --binary | --hex`"
        );
        eprintln!("example `{BIN_NAME} 7`");
        eprintln!();
        eprintln!("{AUTHOR} - 2025");
        return Ok(());
    }

    let value: usize = value.unwrap().parse()?;

    let next = args.next();
    let value = if let Some("--roman") = next.as_deref() {
        format::to_roman_numeral(value)
    } else if let Some("--binary") = next.as_deref() {
        format::to_binary(value)
    } else if let Some("--hex") = next.as_deref() {
        format::to_hex(value)
    } else {
        format::to_english(value)
    };

    println!("{value}");

    Ok(())
}
