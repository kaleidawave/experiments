use crate::{Configuration, Expression};

/// TODO as the application thingy has been changed, this should as act as an intermediatery to help
/// with using the AST
#[derive(Debug)]
pub enum ExpressionKinds<'a> {
	UnaryOperator {
		operator: &'a str,
		operand: Box<ExpressionKinds<'a>>,
		prefix: bool,
	},
	BinaryOperator {
		lhs: Box<ExpressionKinds<'a>>,
		operator: &'a str,
		rhs: Box<ExpressionKinds<'a>>,
	},
	Variable(&'a str),
	Literal(f64),
	Application {
		on: &'a str,
		arguments: Vec<Box<ExpressionKinds<'a>>>,
	},
	Grouping {
		values: Vec<Self>,
	},
}

impl<'a> ExpressionKinds<'a> {
	pub fn from_expression(expression: &Expression<'a>, config: &Configuration) -> Self {
		match expression {
			Expression::Identifier { prefix: _, name } => ExpressionKinds::Variable(name),
			Expression::Grouped { values } => ExpressionKinds::Grouping {
				values: values
					.iter()
					.map(|value| ExpressionKinds::from_expression(value, config))
					.collect(),
			},
			Expression::Application { on, argument } => {
				if let Expression::Identifier { prefix: _, name } = &**on {
					if let Some(operator) =
						config.unary_operators.iter().find(|op| op.representation == *name)
					{
						ExpressionKinds::UnaryOperator {
							operator: operator.representation,
							prefix: operator.prefix,
							operand: Box::new(ExpressionKinds::from_expression(argument, config)),
						}
					} else {
						todo!("{expression:?}")
					}
				} else if let Expression::Application { on, argument: inner_argument } = &**on {
					if let Expression::Identifier { prefix: _, name } = &**on {
						if let Some(operator) =
							config.binary_operators.iter().find(|op| op.representation == *name)
						{
							ExpressionKinds::BinaryOperator {
								operator: operator.representation,
								lhs: Box::new(ExpressionKinds::from_expression(
									inner_argument,
									config,
								)),
								rhs: Box::new(ExpressionKinds::from_expression(argument, config)),
							}
						} else {
							todo!("{expression:?}")
						}
					} else {
						todo!("{expression:?}")
					}
				} else {
					todo!("{expression:?}")
				}
			}
		}
	}
}
