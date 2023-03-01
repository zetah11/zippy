pub mod messages;
pub mod source;

pub trait Db: salsa::DbWithJar<Jar> {}

impl<Database: salsa::DbWithJar<Jar>> Db for Database {}

#[salsa::jar(db = Db)]
pub struct Jar(crate::messages::Messages, crate::source::Source);
