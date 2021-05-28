use crate::Literal;


// We call `expect` in all these impls: this library aims to implement exactly
// the Rust grammar, so if we have a valid Rust literal, we should always be
// able to parse it.
impl From<proc_macro::Literal> for Literal<String> {
    fn from(src: proc_macro::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

impl From<&proc_macro::Literal> for Literal<String> {
    fn from(src: &proc_macro::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Literal> for Literal<String> {
    fn from(src: proc_macro2::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

#[cfg(feature = "proc-macro2")]
impl From<&proc_macro2::Literal> for Literal<String> {
    fn from(src: &proc_macro2::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}
