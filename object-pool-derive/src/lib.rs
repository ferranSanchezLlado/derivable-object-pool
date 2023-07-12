use proc_macro::TokenStream;
use syn::DeriveInput;

fn impl_object_pool_derive_macro(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident_capital = ident.to_string().to_ascii_uppercase();
    let pool = quote::format_ident!("{ident_capital}_OBJECT_POOL");

    let attrs = ast.attrs;

    // 0: use default generator
    // _: example: #[generator(Test2::new_item)]
    let generator = {
            // Find attribute generator
            let attr = attrs
                .iter()
                .find(|attr| attr.path().is_ident("generator"));
            match attr {
                Some(attr) => {
                    let generator = attr.parse_args::<syn::Expr>().unwrap();
                    quote::quote! { #generator }
                }
                None => quote::quote! { #ident::default }
            }
    };


    quote::quote! {
        static #pool: Pool<#ident> = Pool::new(#generator);

        impl #impl_generics ObjectPool for #ident #ty_generics #where_clause {
            fn pool<'a>() -> &'a Pool<Self> {
                &#pool
            }
        }
    }
    .into()
}

#[proc_macro_derive(ObjectPool, attributes(generator))]
pub fn object_pool_derive_macro(tokens: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(tokens).unwrap();

    impl_object_pool_derive_macro(ast)
}
