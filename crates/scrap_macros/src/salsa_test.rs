use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Attribute macro for Salsa-based tests
///
/// Wraps the test function in a Salsa tracked function and creates a database.
/// The test function will be called with the database as the first argument.
///
/// # Example
/// ```
/// #[salsa_test]
/// fn my_test(db: &dyn Db) {
///     // test code here
/// }
/// ```
pub(crate) fn salsa_test_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    // Create the wrapper function name
    let wrapper_name = syn::Ident::new(&format!("{}_impl", fn_name), fn_name.span());

    // Extract function inputs (parameters)
    let fn_inputs = &input_fn.sig.inputs;

    // Generate the expanded code
    let expanded = quote! {
        #[salsa::tracked]
        fn #wrapper_name(#fn_inputs) {
            #fn_body
        }

        #[test]
        #(#fn_attrs)*
        #fn_vis fn #fn_name() {
            let db = scrap_shared::salsa::ScrapDb::default();
            #wrapper_name(&db);
        }
    };

    expanded.into()
}
