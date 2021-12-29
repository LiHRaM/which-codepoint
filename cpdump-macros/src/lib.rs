use cpdump_macros_impl::{map_from_csv_impl, ImplArgs};
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, spanned::Spanned};

struct Args {
    path: String,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: syn::ExprLit = input.parse()?;
        match expr.lit {
            syn::Lit::Str(lit_str) => Ok(Args {
                path: lit_str.value(),
            }),
            _ => Err(syn::Error::new(
                expr.span(),
                "File path must be a string literal",
            )),
        }
    }
}

impl Into<ImplArgs> for Args {
    fn into(self) -> ImplArgs {
        ImplArgs { path: self.path }
    }
}

#[proc_macro]
pub fn map_from_csv(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let output = map_from_csv_impl(args);
    TokenStream::from(output)
}
