use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input};

struct VecItems(pub Vec<syn::ItemEnum>);

impl Parse for VecItems {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            let item: syn::ItemEnum = input.parse()?;
            items.push(item);
        }
        Ok(VecItems(items))
    }
}

#[proc_macro]
pub fn expand_tokens(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as VecItems).0;

    let mut all_variants = Vec::new();
    let mut token_enum = None;

    let items = items
        .into_iter()
        .filter_map(|item| {
            if item.ident == "Token" {
                token_enum.replace(item);
                None
            } else {
                all_variants.extend(item.variants.clone());
                Some(item)
            }
        })
        .map(|mut item| {
            for variant in &mut item.variants {
                variant.attrs.clear();
            }

            item
        })
        .collect::<Vec<_>>();

    if token_enum.is_none() {
        return quote! {
            compile_error!("No Token enum found in the input");
        }
        .into();
    }

    let mut token_enum = token_enum.unwrap();

    for var in all_variants {
        token_enum.variants.push(var);
    }

    quote! {
        #(#items)*

        #token_enum
    }
    .into()
}
