extern crate proc_macro;

use csv::ReaderBuilder;
use proc_macro::TokenStream;
use proc_quote::quote;
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
            _ => Err(syn::Error::new(expr.span(), "File path must be a string")),
        }
    }
}

#[proc_macro]
pub fn csv_file<'a>(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let reader = ReaderBuilder::new()
        .delimiter(b';')
        .comment(Some(b'#'))
        .from_path(&args.path)
        .unwrap();

    let records = read_file(reader);

    TokenStream::from(quote! {
        &[
            #((#(#records),*)),*
        ]
    })
}

fn read_file(mut reader: csv::Reader<std::fs::File>) -> Vec<Vec<String>> {
    reader.deserialize().filter_map(|x| x.ok()).collect()
}
