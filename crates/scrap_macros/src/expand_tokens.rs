use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, LitStr, Meta, parse::Parse, parse_macro_input};

pub(crate) struct VecItems(pub Vec<syn::ItemEnum>);

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

pub(crate) fn expand_tokens_impl(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as VecItems).0;

    let mut all_variants = Vec::new();
    let mut token_enum = None;
    let mut display_arms = Vec::new();

    // Store enum names and their variants for generating is_* methods
    let mut sub_enums: Vec<(syn::Ident, Vec<syn::Ident>)> = Vec::new();

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

                // Store enum name and variant names for is_* methods
                let variant_names: Vec<syn::Ident> =
                    item.variants.iter().map(|v| v.ident.clone()).collect();
                sub_enums.push((item.ident.clone(), variant_names));

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

    let all_name = all_variants.iter().map(|v| &v.ident).collect::<Vec<_>>();

    // Generate is_* methods for each sub-enum
    let is_methods = sub_enums.iter().map(|(enum_name, variant_names)| {
        // Convert enum name to snake_case for method name (e.g., BinaryOperators -> is_binary_operators)
        let method_name = syn::Ident::new(
            &format!("is_{}", enum_name.to_string().to_snake_case()),
            enum_name.span(),
        );

        quote! {
            pub fn #method_name(&self) -> bool {
                matches!(self, #(Token::#variant_names)|*)
            }
        }
    });

    // Generate TryFrom implementations for each sub-enum
    let try_from_impls = sub_enums.iter().map(|(enum_name, variant_names)| {
        let match_arms = variant_names.iter().map(|variant| {
            quote! {
                Token::#variant => Ok(#enum_name::#variant)
            }
        });

        quote! {
            impl TryFrom<Token> for #enum_name {
                type Error = Token;

                fn try_from(token: Token) -> Result<Self, Self::Error> {
                    match token {
                        #(#match_arms,)*
                        other => Err(other),
                    }
                }
            }

            impl<'a> TryFrom<&'a Token> for #enum_name {
                type Error = &'a Token;

                fn try_from(token: &'a Token) -> Result<Self, Self::Error> {
                    match token {
                        #(Token::#variant_names => Ok(#enum_name::#variant_names),)*
                        other => Err(other),
                    }
                }
            }
        }
    });

    let from_u32 = quote! {
        impl Token {
            pub fn from_u32(val: u32) -> Token {
                match val {
                    #(x if x == Token::#all_name as u32 => Token::#all_name,)*
                    _ => panic!("unhandled value: {}", val),
                }
            }

            #(#is_methods)*
        }
    };

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

    // Remove processed attributes from Token enum variants but keep enum-level attributes
    for variant in &mut token_enum.variants {
        variant
            .attrs
            .retain(|attr| !attr.path().is_ident("display"));
    }

    quote! {
        #(#items)*

        #token_enum

        #display_impl

        #from_u32

        #(#try_from_impls)*
    }
    .into()
}
