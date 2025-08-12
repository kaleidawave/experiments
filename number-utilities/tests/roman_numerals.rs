use number_utilities::format::to_roman_numeral;
use number_utilities::parse::parse_roman_numeral;

#[test]
fn upto_mmmcmxcix() {
    for value in 0..4000 {
        let roman_numeral = to_roman_numeral(value);
        let out_value = parse_roman_numeral(&roman_numeral);
        assert_eq!(value, out_value);
    }
}

#[test]
fn format_roman_numeral() {
    assert_eq!(to_roman_numeral(39), "XXXIX");
    assert_eq!(to_roman_numeral(246), "CCXLVI");
    assert_eq!(to_roman_numeral(789), "DCCLXXXIX");
    assert_eq!(to_roman_numeral(2421), "MMCDXXI");

    assert_eq!(to_roman_numeral(160), "CLX");
    assert_eq!(to_roman_numeral(207), "CCVII");
    assert_eq!(to_roman_numeral(1009), "MIX");
    assert_eq!(to_roman_numeral(1066), "MLXVI");

    assert_eq!(to_roman_numeral(1776), "MDCCLXXVI");
    assert_eq!(to_roman_numeral(1918), "MCMXVIII");
    assert_eq!(to_roman_numeral(1944), "MCMXLIV");
    assert_eq!(to_roman_numeral(2025), "MMXXV");

    assert_eq!(to_roman_numeral(3999), "MMMCMXCIX");
}
