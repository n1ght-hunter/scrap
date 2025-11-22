use proc_macro::TokenStream;

mod expand_tokens;
mod salsa_test;

#[proc_macro]
pub fn expand_tokens(input: TokenStream) -> TokenStream {
    expand_tokens::expand_tokens_impl(input)
}

#[proc_macro_attribute]
pub fn salsa_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    salsa_test::salsa_test_impl(attr, item)
}
