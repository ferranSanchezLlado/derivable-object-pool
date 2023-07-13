//! # Derivable Object Pool Macro
//!
//! Internal crate for [derivable-object-pool](https://crates.io/crates/derivable-object-pool)
//! crate. That provides a derive macro for [`ObjectPool`] trait.
//!
//! [`ObjectPool`]: trait.ObjectPool.html
use proc_macro::TokenStream;
use syn::DeriveInput;

fn impl_object_pool_derive_macro(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident_capital = ident.to_string().to_ascii_uppercase();
    let pool = quote::format_ident!("{ident_capital}_OBJECT_POOL");

    let attrs = ast.attrs;

    let generator = {
        // Find attribute generator
        let attr = attrs.iter().find(|attr| attr.path().is_ident("generator"));
        match attr {
            Some(attr) => {
                let generator = attr.parse_args::<syn::Expr>().unwrap();
                quote::quote! { #generator }
            }
            None => quote::quote! { #ident::default },
        }
    };

    quote::quote! {
        static #pool: Pool<#ident> = Pool::new(#generator);

        impl #impl_generics ObjectPool for #ident #ty_generics #where_clause {
            #[inline]
            fn pool<'a>() -> &'a Pool<Self> {
                &#pool
            }
        }
    }
    .into()
}

/// Derive macro for [`ObjectPool`] trait implementation.
/// Optionally, you can specify a generator function for the pool. If not
/// specified, the trait will try to use [`Default`] trait implementation.
///
/// Internally, the macro will generate a static variable with the name of
/// `#[ident]_OBJECT_POOL` (where `#[ident]` is the name of the type in
/// uppercase) and implement the [ObjectPool] trait for the type.
///
/// # Example
///
/// ```rust
/// use derivable_object_pool::prelude::*;
///
/// #[derive(ObjectPool)]
/// #[generator(Test2::new_item)]
/// struct Test2(u32);
///
/// impl Test2 {
///     fn new_item() -> Self {
///         Test2(10)
///     }
/// }
///
/// fn main() {
///     let mut item = Test2::new();
///     item.0 = 20;
///     assert_eq!(item.0, 20);
///     drop(item);
///     assert_eq!(Test2::pool().len(), 1);
/// }
/// ```
///
/// Generated extra code for the derive macro:
///
/// ```rust
///# use derivable_object_pool::prelude::*;
///#
///# struct Test2(u32);
///#
///# impl Test2 {
///#     fn new_item() -> Self {
///#         Test2(10)
///#     }
///# }
///#
/// static TEST2_OBJECT_POOL: Pool<Test2> = Pool::new(Test2::new_item);
///
/// impl ObjectPool for Test2 {
///     #[inline]
///     fn pool<'a>() -> &'a Pool<Self> {
///         &TEST2_OBJECT_POOL
///     }
/// }
///#
///# fn main() {
///#     let mut item = Test2::new();
///#     item.0 = 20;
///#     assert_eq!(item.0, 20);
///#     drop(item);
///#     assert_eq!(Test2::pool().len(), 1);
///# }
/// ```
///
/// # Attributes
///
/// ## generator
///
/// Specify a generator function for the pool. If not specified, the trait will
/// try to use [`Default`] trait implementation.
///
///
/// [`ObjectPool`]: trait.ObjectPool.html
/// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
#[proc_macro_derive(ObjectPool, attributes(generator))]
pub fn object_pool_derive_macro(tokens: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(tokens).unwrap();

    impl_object_pool_derive_macro(ast)
}
