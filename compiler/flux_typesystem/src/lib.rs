mod check;
mod constraint;
pub mod diagnostics;
mod env;
mod intern;
mod name_res;
pub mod r#type;

pub use check::TChecker;
pub use constraint::Constraint;
pub use env::TEnv;
pub use r#type::{ConcreteKind, Type, TypeId, TypeKind};