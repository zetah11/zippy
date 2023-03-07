#[salsa::interned]
pub struct StringLiteral {
    #[return_ref]
    pub literal: String,
}

#[salsa::interned]
pub struct NumberLiteral {
    #[return_ref]
    pub literal: String,
}
