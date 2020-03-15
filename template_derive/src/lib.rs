use heck::SnekCase as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/** Derives `Template` for an enum

    * the type must be an enum with named variants, or fieldless variants
    * the fields in the named variants must not be rust identifiers
    * the types in the named variants must implement `std::fmt::Display`
*/
#[proc_macro_derive(Template, attributes(namespace))]
pub fn template(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let syn::DeriveInput {
        ident,
        generics,
        data,
        mut attrs,
        ..
    } = derive_input;

    if attrs.len() != 1 {
        panic!("a namespace attribute with name must be supplied")
    }
    let namespace = match attrs.remove(0).parse_args::<syn::Lit>() {
        Ok(syn::Lit::Str(namespace)) => namespace.value(),
        _ => panic!("a string must be used as a namespace identifier"),
    };

    let variants = match data {
        syn::Data::Enum(e) => get_variants(e.variants.into_iter()).collect::<Vec<_>>(),
        _ => panic!("only enums are allowed"),
    };

    let matches = variants.clone().into_iter()
        .map(|(var, fields)| (var, fields.into_iter().filter_map(|v| v.ident)))
        .map(|(var, fields)| {
            let args = fields.clone().map(|v| {
                let k = v.to_string();
                quote! { with(#k, #v) }
            });
            quote! {
                #var { #(#fields),* } => {
                    let args = markings::Args::new()#(.#args)*.build();
                    let opts = markings::Opts::default().optional_keys().duplicate_keys().empty_template().build();
                    let template = markings::Template::parse(template, opts).ok()?;
                    template.apply(&args).ok()
                }
            }
        });

    let names = variants.into_iter().map(|(var, _)| {
        let name = var.to_string().to_snek_case();
        quote! { #var { .. } => #name }
    });
    let name = ident.to_string().to_snek_case();
    let namespace = namespace.to_snek_case();

    let ast = quote! {
        impl #generics Template for #ident #generics {
            fn namespace() -> &'static str { #namespace }

            fn name() -> &'static str { #name }

            fn variant(&self) -> &'static str {
                use #ident::*;
                match self {
                    #(#names),*
                }
            }

            fn apply(&self, template: &str) -> Option<String> {
                use #ident::*;
                match self {
                    #(#matches),*
                }
            }
        }
    };
    ast.into()
}

fn get_variants(
    variants: impl Iterator<Item = syn::Variant>,
) -> impl Iterator<Item = (syn::Ident, Vec<syn::Field>)> {
    variants.map(|var| {
        let ident = var.ident;
        let fields = match var.fields {
            syn::Fields::Named(fields) => fields,
            syn::Fields::Unit => return (ident, vec![]),
            _ => panic!("only named fields are allowed"),
        };
        if fields.named.is_empty() {
            panic!("named variants must have fields")
        }
        (ident, fields.named.into_iter().collect())
    })
}
