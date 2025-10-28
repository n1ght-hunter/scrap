use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, LitStr, Meta, parse::Parse, parse_macro_input};

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

fn extract_token_string(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("token") {
            if let Ok(Meta::List(meta_list)) = attr.meta.clone().try_into() {
                if let Ok(lit_str) = syn::parse2::<LitStr>(meta_list.tokens) {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn extract_display_string(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("display") {
            if let Ok(Meta::List(meta_list)) = attr.meta.clone().try_into() {
                if let Ok(lit_str) = syn::parse2::<LitStr>(meta_list.tokens) {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn create_display_arm(
    variant_name: &syn::Ident,
    variant_fields: &syn::Fields,
    variant_attrs: &[syn::Attribute],
) -> Option<proc_macro2::TokenStream> {
    match variant_fields {
        syn::Fields::Unit => {
            if let Some(token_str) = extract_token_string(variant_attrs) {
                // Escape braces in token strings for format strings
                let escaped_token = token_str.replace("{", "{{").replace("}", "}}");
                Some(quote! {
                    Token::#variant_name => write!(f, #escaped_token)
                })
            } else if let Some(display_str) = extract_display_string(variant_attrs) {
                Some(quote! {
                    Token::#variant_name => write!(f, #display_str)
                })
            } else {
                None
            }
        }
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            // For single field variants like Str(&str), Int(i64), use the field value
            Some(quote! {
                Token::#variant_name(val) => write!(f, "{}", val)
            })
        }
        _ => {
            // For more complex fields, use custom display if available
            if let Some(display_str) = extract_display_string(variant_attrs) {
                Some(quote! {
                    Token::#variant_name(..) => write!(f, #display_str)
                })
            } else {
                None
            }
        }
    }
}

#[proc_macro]
pub fn expand_tokens(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as VecItems).0;

    let mut all_variants = Vec::new();
    let mut token_enum = None;
    let mut display_arms = Vec::new();

    let items = items
        .into_iter()
        .filter_map(|item| {
            if item.ident == "Token" {
                // Extract existing display arms from Token enum if any
                for variant in &item.variants {
                    if let Some(display_arm) =
                        create_display_arm(&variant.ident, &variant.fields, &variant.attrs)
                    {
                        display_arms.push(display_arm);
                    }
                }

                token_enum.replace(item);
                None
            } else {
                // Process other enums and collect display information
                for variant in &item.variants {
                    if let Some(display_arm) =
                        create_display_arm(&variant.ident, &variant.fields, &variant.attrs)
                    {
                        display_arms.push(display_arm);
                    }
                }

                all_variants.extend(item.variants.clone());
                Some(item)
            }
        })
        .map(|mut item| {
            const REMOVE_ATTRS: &[&str] = &["token", "display", "regex"];
            for variant in &mut item.variants {
                // Remove ALL attributes as they're processed separately
                variant
                    .attrs
                    .retain(|attr| REMOVE_ATTRS.iter().all(|&id| !attr.path().is_ident(id)));
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

    // Remove processed attributes from Token enum variants but keep enum-level attributes
    for variant in &mut token_enum.variants {
        variant
            .attrs
            .retain(|attr| !attr.path().is_ident("display"));
    }

    for var in all_variants {
        token_enum.variants.push(var);
    }

    // Generate Display implementation
    let display_impl = if !display_arms.is_empty() {
        quote! {
            impl std::fmt::Display for Token {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_arms,)*
                        _ => write!(f, "{:?}", self),
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #(#items)*

        #token_enum

        #display_impl
    }
    .into()
}
