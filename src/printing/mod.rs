use quote::{ToTokens, TokenStreamExt};

use crate::Buffer;

macro_rules! to_tokens_simple {
    ($id:ident) => {
        impl<B: Buffer + Clone> ToTokens for crate::$id<B> {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.append(<Self as Into<proc_macro2::Literal>>::into(self.clone()))
            }

            fn into_token_stream(self) -> proc_macro2::TokenStream
            where
                Self: Sized,
            {
                proc_macro2::TokenStream::from(proc_macro2::TokenTree::from(<Self as Into<
                    proc_macro2::Literal,
                >>::into(self)))
            }
        }
    };
}
to_tokens_simple!(ByteLit);
to_tokens_simple!(ByteStringLit);
to_tokens_simple!(CharLit);
to_tokens_simple!(FloatLit);
to_tokens_simple!(IntegerLit);
to_tokens_simple!(StringLit);

impl ToTokens for crate::BoolLit {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use crate::BoolLit::*;
        tokens.append(proc_macro2::Ident::new(
            match self {
                True => "true",
                False => "false",
            },
            proc_macro2::Span::call_site(),
        ))
    }
}

#[cfg(test)]
mod tests;
