use general_parser::{Configuration, Expression}; // BinaryOperator,

fn main() {
	let allocator = bumpalo::Bump::new();

	let expression = r#"(module
  (func (result i32)
    (i32.const 42)
  )
  (export "forty_two" (func 0))
)"#;

	let configuration = Configuration::default();

	let expression: Expression<WASMValue> =
		Expression::from_string(expression, &configuration, &allocator);

	dbg!(expression);
}

#[derive(Debug)]
pub enum WASMValue<'a> {
	String(&'a str),
	Index(usize),
	Float(f64),
	Keyword(&'a str),
	Instruction(Instruction<'a>),
}

#[derive(Debug)]
pub struct Instruction<'a>(pub &'a str);

impl<'a> general_parser::Literal<'a> for WASMValue<'a> {
	fn from_str(on: &'a str) -> Self {
		if let Ok(idx) = on.parse::<usize>() {
			Self::Index(idx)
		} else if let Ok(idx) = on.parse::<f64>() {
			Self::Float(idx)
		} else if let Some(on) = on.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')) {
			Self::String(on)
		} else if on.contains('.') {
			// TODO check
			Self::Instruction(Instruction(on))
		} else {
			Self::Keyword(on)
		}
	}
}
