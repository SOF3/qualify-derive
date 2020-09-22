extern crate proc_macro;

use proc_macro2::{TokenStream, TokenTree};
#[doc(hidden)]
pub use quote::quote;

/// Entry function to call from the `proc_macro_attribute` function.
///
/// Pay attention to the mixed use of `proc_macro::TokenStream` and `proc_macro2::TokenStream` in
/// the signature.
///
/// See the [README](https://github.com/SOF3/qualify-derive) for detailed explanation.
pub fn fix(attr: proc_macro::TokenStream, item: proc_macro::TokenStream, target: TokenStream, imports: &[TokenStream], passthru: Option<TokenStream>) -> proc_macro::TokenStream {
    fix_(attr.into(), item.into(), target, imports, passthru).unwrap_or_else(|err| err.to_compile_error()).into()
}

fn fix_(attr: TokenStream, item: TokenStream, target: TokenStream, imports: &[TokenStream], passthru: Option<TokenStream>) -> syn::Result<TokenStream> {
    let mut item = syn::parse2::<syn::DeriveInput>(item)?;
    let vis = item.vis.clone();
    item.vis = syn::parse2(quote!(pub(super))).unwrap();
    let ident = &item.ident;

    let unused = item.attrs.iter().any(|attr| attr.path.is_ident("allow") && {
        for token in attr.tokens.clone() {
            if let TokenTree::Group(group) = token {
                for token in group.stream() {
                    if let TokenTree::Ident(ident) = token {
                        if ident == "unused" || ident == "unused_imports" || ident == "dead_code" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    });
    let unused = if unused {
        quote!(#[allow(unused_imports)])
    } else {
        quote!()
    };

    let mod_name = quote::format_ident!("__qualify_derive_{}", ident);

    let passthru = passthru.and_then(|attr_name| {
        if attr.is_empty() {
            None
        } else {
            Some(quote!(#[#attr_name(#attr)]))
        }
    });

    let output = quote! {
        #[allow(non_snake_case)]
        mod #mod_name {
            use super::*;
            #(
                #[allow(unused_imports)]
                use #imports;
            )*
            #[derive(#target)]
            #passthru
            #item
        }
        #unused
        #vis use #mod_name::#ident;
    };
    Ok(output)
}

/// Convenient macro to declare an attribute that calls `fix`.
///
/// See the [README](https://github.com/SOF3/qualify-derive) for detailed explanation.
#[macro_export]
macro_rules! declare {
    ($(#[$meta:meta])* $name:ident derives $target:ty; $(use $imports:ty;)*) => {
        $crate::declare!(@INTERNAL $(#[$meta])* $name; $target; $($imports),*; None);
    };
    ($(#[$meta:meta])* $name:ident derives $target:ty; $(use $imports:ty;)* attr $attr:ident $(;)?) => {
        $crate::declare!(@INTERNAL $(#[$meta])* $name; $target; $($imports),*; Some($crate::quote!($attr)));
    };
    (@INTERNAL $(#[$meta:meta])* $name:ident; $target:ty; $($imports:ty),*; $passthru:expr) => {
        #[proc_macro_attribute]
        $(#[$meta])*
        pub fn $name(attr: ::proc_macro::TokenStream, item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            use $crate::quote;
            $crate::fix(attr, item, quote!($target), &[$(quote!($imports)),*], $passthru)
        }
    };
}
