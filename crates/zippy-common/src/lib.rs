pub mod hir;
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
pub struct Jar(crate::names2::Name);

pub trait Db: DbWithJar<Jar> {}

impl<DB> Db for DB where DB: salsa::DbWithJar<Jar> {}
