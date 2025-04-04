use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[cfg(feature = "ast")]
#[proc_macro_derive(Spanned)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident {
            pub fn span(&self) -> &crate::Span {
                &self.span
            }
        }
    };
    output.into()
}
