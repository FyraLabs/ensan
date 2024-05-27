use std::collections::HashSet;

use syn::{parse::Parse, punctuated::Punctuated, Expr, Ident, Token};

struct Args {
    params: HashSet<Expr>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            params: vars.into_iter().collect(),
        })
    }
}
