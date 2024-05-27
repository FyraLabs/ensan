use proc_macro::TokenStream;
use quote::ToTokens;
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
        let mut params = ensan_attr.args;
        for param in &mut params {
            let syn::Expr::Call(ecall) = param else {
                let syn::Expr::Path(_) = param else {
                    return syn::Error::new(param.span(), "not a call/path syntax")
                        .to_compile_error()
                        .into();
                };
                continue;
            };
            {
                // old code for testing?
                // WARN: remove this
                let mut args = ecall.args.iter_mut().collect::<Vec<_>>();
                while !args.is_empty() {
                    let newvar = args.as_mut_slice();
                    let [one] = newvar else {
                        return syn::Error::new(
                            args[1].span(),
                            "This call pattern has more than 1 arguments which is not allowed",
                        )
                        .to_compile_error()
                        .into();
                    };
                    let rewrite = quote::quote! {
                        Box::new(::hcl::eval::ParamType::#one)
                    }
                    .into();
                    **one = syn::parse_macro_input!(rewrite as Expr);
                    let syn::Expr::Call(ecall) = one else {
                        unreachable!()
                    };
                    let inner = &mut ecall.args[0];
                    let syn::Expr::Call(ecall) = inner else {
                        let syn::Expr::Path(epath) = inner else {
                            return syn::Error::new(param.span(), "not a call/path syntax")
                                .to_compile_error()
                                .into();
                        };
                        let rewrite = quote::quote! {
                            ::hcl::eval::ParamType::#epath
                        }
                        .into_token_stream()
                        .into();
                        *epath = syn::parse_macro_input!(rewrite as syn::ExprPath);
                        break;
                    };
                    args = ecall.args.iter_mut().collect::<Vec<_>>();
                }
            }
            if ecall.args.len() != 1 {
                return syn::Error::new(
                    ecall.args[1].span(),
                    "This call pattern has more than 1 arguments which is not allowed",
                )
                .to_compile_error()
                .into();
            }
            let mut i = 0;
            let mut args = ecall.args.iter_mut().collect::<Vec<_>>();
            while let [rest @ .., one] = args.as_mut_slice() {
                if rest.len() != i {
                    return syn::Error::new(
                        rest.last().span(),
                        "This call pattern has more than 1 arguments which is not allowed",
                    )
                    .to_compile_error()
                    .into();
                }
                i += 1;
                let rewrite = quote::quote! {
                    Box::new(::hcl::eval::ParamType::#one)
                }
                .into();
                **one = syn::parse_macro_input!(rewrite as Expr);
                let Expr::Call(ecall) = one else {
                    unreachable!()
                };
                let inner = &mut ecall.args[0];
                let Expr::Call(ecall) = inner else {
                    let Expr::Path(epath) = inner else {
                        return syn::Error::new(param.span(), "not a call/path syntax")
                            .to_compile_error()
                            .into();
                    };
                    let rewrite = quote::quote! {
                        ::hcl::eval::ParamType::#epath
                    }
                    .into_token_stream()
                    .into();
                    *epath = syn::parse_macro_input!(rewrite as syn::ExprPath);
                    break;
                };
                args = ecall.args.iter_mut().collect::<Vec<_>>();
            }
        }
        // let params = params.iter().map(|param| {
        //     let sstream = param.to_token_stream().to_string();
        //     sstream
        //         .replace('(', "(Box::new(::hcl::eval::ParamType::")
        //         .replace(')', "))")
        //         .to_token_stream()
        // });
        // let params = params.collect::<Vec<_>>();
        // println!("{:?}", params);
        declare_func_stmts.push(quote::quote! {
            ctx.declare_func(stringify!(#fname), ::hcl::eval::FuncDef::new(#fname, [#(::hcl::eval::ParamType::#params),*]));
        });
    }
    let out: TokenStream = quote::quote! {
        #input
        pub fn #new_fn_name(ctx: &mut ::hcl::eval::Context) {
            use #mod_name::*;
            use ::hcl::eval::ParamType::*;
            #(#declare_func_stmts)*
        }
    }
    .into();
    println!("{}", out.to_string());
    out
}
