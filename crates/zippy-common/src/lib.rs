pub mod hir;
pub mod hir2;
pub mod kinds;
pub mod message;
pub mod mir;
pub mod names;
pub mod names2;
pub mod sizes;
pub mod thir;

pub use malachite::Rational as Number;
use salsa::DbWithJar;

pub use self::driver::{Driver, EvalAmount, IrOutput};

mod driver;

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::names2::Name,
    crate::hir2::Decls,
    crate::hir2::Definitions,
    crate::hir2::TypeckResult,
);

pub trait Db: DbWithJar<Jar> {}

impl<DB> Db for DB where DB: salsa::DbWithJar<Jar> {}
