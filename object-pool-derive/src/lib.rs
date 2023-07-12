use proc_macro::TokenStream;
use syn::DeriveInput;

fn impl_object_pool_derive_macro(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let ident_capital = ident.to_string().to_ascii_uppercase();
    let pool = quote::format_ident!("{ident_capital}_OBJECT_POOL");

    quote::quote! {
        static #pool: Pool<#ident> = Pool::new();

        impl ObjectPool for #ident {
            fn pool<'a>() -> &'a Pool<#ident> {
                &#pool
            }
        }
    }
    .into()
}

#[proc_macro_derive(ObjectPool)]
pub fn object_pool_derive_macro(tokens: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(tokens).unwrap();

    impl_object_pool_derive_macro(ast)
}
