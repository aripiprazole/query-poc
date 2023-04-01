use proc_macro::TokenStream;

use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed, Ident, Index,
    Member, Type,
};

#[proc_macro_derive(Query, attributes(output, query))]
pub fn query(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let query_fn = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("query"))
        .map(|attr| attr.parse_args::<Ident>())
        .ok_or_else(|| {
            let err = Error::new_spanned(input.clone(), "Missing query attribute");
            panic!("{err}")
        })
        .unwrap()
        .unwrap();

    let output = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("output"))
        .map(|attr| attr.parse_args::<Type>())
        .ok_or_else(|| {
            let err = Error::new_spanned(input.clone(), "Missing output attribute");
            panic!("{err}")
        })
        .unwrap()
        .unwrap();

    let struct_data = match input.data {
        syn::Data::Struct(data) => data,
        _ => {
            let err = Error::new_spanned(input, "Only structs can be queries");
            panic!("{err}")
        }
    };

    let name = input.ident;
    let mut hash_fields = Vec::new();
    match struct_data.fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            for i in 0..unnamed.len() {
                hash_fields.push(Member::Unnamed(Index::from(i)));
            }
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            for n in named {
                hash_fields.push(Member::Named(n.ident.unwrap()));
            }
        }
        Fields::Unit => {}
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #(self.#hash_fields.hash(state));*
            }
        }

        impl crate::eszopiclone::Query for #name {
            type Output = #output;

            fn execute(&self, compiler: &mut crate::eszopiclone::Compiler) -> crate::eszopiclone::Result<#output> {
                #query_fn(compiler, self)
            }
        }
        // ...
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
