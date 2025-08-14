use number_utilities::format;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let value = args.next();
    if let Some("--help") | None = value.as_deref() {
        eprintln!(
            "usage: `{BIN_NAME} *number* (decinary)` or `{BIN_NAME} *number* (--roman | --binary | --hex | --denary | --english) | --seperator *seperator* (default ` ` if specififed)`"
        );
        eprintln!("example `{BIN_NAME} 7`");
        eprintln!();
        eprintln!("{AUTHOR} - 2025");
        return Ok(());
    }

    let value: usize = value.unwrap().parse()?;

    let next = args.next();

    let s;
    let seperator = if let Some("--seperator") = args.next().as_deref() {
        s = args.next();
        s.as_deref().unwrap_or(" ")
    } else {
        ""
    };

    let value = match next.as_deref() {
        Some("--roman") => format::to_roman_numeral(value),
        Some("--binary") => format::to_binary(value),
        Some("--hex") => format::to_hex(value, seperator),
        Some("--denary") => format::to_denary(value, seperator),
        Some("--english") | None => format::to_english(value),
        Some(format) => {
            return Err(format!("unknown format {format:?}. expected --roman, --binary, --hex, --denary or --english").into());
        }
    };

    println!("{value}");

    Ok(())
}
