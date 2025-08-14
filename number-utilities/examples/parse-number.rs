use number_utilities::parse;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let value = args.next();
    if let Some("--help") | None = value.as_deref() {
        eprintln!(
            "usage: `{BIN_NAME} *number*` or `{BIN_NAME} *number* (--roman | --binary | --hex | --english)`"
        );
        eprintln!("examples: `{BIN_NAME} seven`, `{BIN_NAME} IX --roman`");
        eprintln!();
        eprintln!("{AUTHOR} - 2025");
        return Ok(());
    }

    let value = value.unwrap();

    let next = args.next();
    let format = next.as_deref().unwrap_or("--english");
    let result = match format {
        "--roman" => parse::parse_roman_numeral(&value),
        "--binary" => parse::parse_binary(&value),
        "--hex" => parse::parse_hex(&value),
        "--english" => parse::parse_english(&value).map_err(|_| ()),
        format => {
            return Err(format!(
                "unknown format {format}. expected --roman, --binary, --hex or --english"
            )
            .into());
        }
    };

    if let Ok(value) = result {
        println!("{value}");
        Ok(())
    } else {
        return Err(format!("could not format {value} in format {format}").into());
    }
}
