#![feature(try_trait_v2)]
#![doc = include_str!("README.md")]

use std::ops::{ControlFlow, FromResidual, Try};

#[derive(PartialEq, Eq, Default, Debug, Clone, Copy)]
pub enum Early<T> {
    Some(T),
    #[default]
    None,
}

// https://doc.rust-lang.org/stable/std/ops/trait.FromResidual.html
impl<T> FromResidual for Early<T> {
    fn from_residual(residual: Early<T>) -> Self {
        residual
    }
}

// https://doc.rust-lang.org/stable/std/ops/trait.Try.html
impl<T> Try for Early<T> {
    type Output = ();
    type Residual = Early<T>;

    fn from_output((): Self::Output) -> Self {
        Early::None
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Early::Some(v) => ControlFlow::Break(Early::Some(v)),
            Early::None => ControlFlow::Continue(()),
        }
    }
}
