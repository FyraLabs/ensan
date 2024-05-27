use proc_macro::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Expr, Token};

struct EnsanFnAttrArgs {
    args: Vec<Expr>,
}

impl Parse for EnsanFnAttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let vars = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            args: vars.into_iter().collect(),
        })
    }
}

fn mutate_tokens(args: &[&Expr]) -> proc_macro2::TokenStream {
    if args.is_empty() {
        return quote::quote! {
            compile_error!("empty call syntax in #[ensan_fn(...)]");
        };
    }
    let [arg] = args else {
        return syn::Error::new(
            args[1].span(),
            "Call pattern has >=1 argument which is illegal",
        )
        .into_compile_error();
    };

    if let Expr::Path(epath) = arg {
        return quote::quote! { ::hcl::eval::ParamType::#epath };
    }
    let Expr::Call(ecall) = arg else {
        return syn::Error::new(arg.span(), "not a call/path syntax").into_compile_error();
    };
    let Expr::Path(t) = &*ecall.func else {
        return syn::Error::new(ecall.span(), "not a proper call ident").into_compile_error();
    };

    let inner = mutate_tokens(&ecall.args.iter().collect::<Vec<_>>());

    quote::quote! {
        ::hcl::eval::ParamType::#t(::std::boxed::Box::new(#inner))
    }
}

/// Save the FuncDef into a global variable "ENSAN_INTERNAL_FNS"
#[proc_macro_attribute]
pub fn ensan_internal_fn_mod(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::ItemMod);
    let new_fn_name = syn::parse_macro_input!(args as syn::Ident);
    let mod_name = &input.ident;
    let mut declare_func_stmts = vec![];
    for elm in &mut input.content.iter_mut().next().unwrap().1 {
        let syn::Item::Fn(f) = elm else { continue };
        let fname = &f.sig.ident;
        // ensure it is #[ensan_fn]
        let Some((attr_index, _)) =
            (f.attrs.iter().enumerate()).find(|(_, a)| a.path().is_ident("ensan_fn"))
        else {
            continue;
        };
        let ensan_attr = f
            .attrs
            .swap_remove(attr_index)
            .parse_args()
            .unwrap_or_else(syn::Error::into_compile_error)
            .into();
        let ensan_attr = syn::parse_macro_input!(ensan_attr as EnsanFnAttrArgs);
        let params = ensan_attr.args.iter().map(|param| mutate_tokens(&[param]));
        declare_func_stmts.push(quote::quote! {
            ctx.declare_func(stringify!(#fname), ::hcl::eval::FuncDef::new(#fname, [#(#params),*]));
        });
    }
    quote::quote! {
        #input
        pub fn #new_fn_name(ctx: &mut ::hcl::eval::Context) {
            use #mod_name::*;
            #(#declare_func_stmts)*
        }
    }
    .into()
}
