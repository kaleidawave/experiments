use date_utilities::{FORMATS, Instant};

fn main() {
    let arg = std::env::args().nth(1);
    let format = arg.as_deref().unwrap_or_default();

    let format = match format {
        "FULL_MINIMAL" | "" => FORMATS::FULL_MINIMAL,
        "DATE_MONTH_YEAR" => FORMATS::DATE_MONTH_YEAR,
        "MONTH_DATE_YEAR" => FORMATS::MONTH_DATE_YEAR,
        "DATE_NAME_MONTH_YEAR" => FORMATS::DATE_NAME_MONTH_YEAR,
        "TIME_DATE_NAME_MONTH_YEAR" => FORMATS::TIME_DATE_NAME_MONTH_YEAR,
        "TIME" => FORMATS::TIME,
        "TIME_WITH_SECONDS" => FORMATS::TIME_WITH_SECONDS,
        format => format,
    };

    let now = Instant::now();
    let formatted = now.format(format);
    println!("{formatted}");
}
