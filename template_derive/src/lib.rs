use heck::SnekCase as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error};

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

    if attrs.is_empty() {
        let mut err = Error::new_spanned(
            quote! { attrs},
            "A `namespace` attribute with the template name must be supplied.",
        );
        err.combine(Error::new_spanned(ident, "for this type"));
        return err.to_compile_error().into();
    }

    let attr = attrs.remove(0);
    let namespace = match find_namespace(&attr) {
        Ok(namespace) => namespace.value(),
        Err(err) => return err.to_compile_error().into(),
    };

    let variants = match build_variant_map(data, attr) {
        Ok(variants) => variants,
        Err(err) => return err.to_compile_error().into(),
    };

    let matches = variants.clone().into_iter()
        .map(|(var, fields)| (var, fields.into_iter().filter_map(|v| v.ident)))
        .map(|(var, fields)| {
            let args = fields.clone().map(|v| {
                let k = v.to_string();
                quote! { with(#k, #v) }
            });
            quote! {
                #ident::#var { #(#fields),* } => {
                    let args = template::markings::Args::new()#(.#args)*;
                    let opts = template::markings::Opts::default().optional_keys().duplicate_keys().empty_template().build();
                    let template = template::markings::Template::parse(template, opts).ok()?;
                    template.apply(&args).ok()
                }
            }
        });

    let names_original = variants.iter().map(|(var, _)| {
        let name = var.to_string();
        quote! { #ident::#var { .. } => #name }
    });

    let names = variants.iter().map(|(var, _)| {
        let name = var.to_string().to_snek_case();
        quote! { #ident::#var { .. } => #name }
    });

    let name_original = ident.to_string();
    let name = name_original.to_snek_case();

    let namespace_original = namespace;
    let namespace = namespace_original.to_snek_case();

    let ast = quote! {
        impl #generics template::Template for #ident #generics {
            fn namespace(casing: template::NameCasing) -> &'static str {
                match casing {
                    template::NameCasing::Snake => { #namespace }
                    template::NameCasing::Original => { #namespace_original }
                    _ => unimplemented!()
                }
            }

            fn name(casing: template::NameCasing) -> &'static str {
                match casing {
                    template::NameCasing::Snake => { #name }
                    template::NameCasing::Original => { #name_original }
                    _ => unimplemented!()
                }
            }

            fn variant(&self,casing: template::NameCasing) -> &'static str {
                match casing {
                    template::NameCasing::Snake => { match self { #(#names),* } }
                    template::NameCasing::Original => { match self { #(#names_original),* } }
                    _ => unimplemented!()
                }
            }

            fn apply(&self, template: &str) -> Option<String> {
                match self { #(#matches),* }
            }
        }
    };
    ast.into()
}

fn find_namespace(attr: &syn::Attribute) -> Result<syn::LitStr, syn::Error> {
    let ns = match attr.parse_args::<syn::Lit>() {
        Ok(syn::Lit::Str(namespace)) => namespace,
        Ok(attr) => {
            return Err(Error::new(
                attr.span(), //
                "A string literal must be used as a `namespace` identifier.",
            ));
        }
        // TODO say we cannot parse the name into a Lit (when can this happen?)
        Err(err) => return Err(Error::new(attr.span(), err)),
    };

    let namespace = ns.value();

    if namespace.chars().take_while(|c| !c.is_alphabetic()).count() > 0 {
        return Err(Error::new(
            ns.span(),
            "The namespace must start with a character.",
        ));
    }

    if !namespace
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(Error::new(
            ns.span(),
            "The namespace should be of [A..Z, _, 1..9]",
        ));
    }

    if namespace.chars().any(|c| c.is_whitespace()) {
        return Err(Error::new(
            ns.span(),
            "The namespace cannot contain spaces.",
        ));
    }

    Ok(ns)
}

fn build_variant_map(
    data: syn::Data,
    attr: syn::Attribute,
) -> Result<Vec<(syn::Ident, Vec<syn::Field>)>, syn::Error> {
    let variants = match data {
        syn::Data::Enum(e) if !e.variants.is_empty() => e.variants,
        syn::Data::Enum(e) => {
            return Err(Error::new(
                e.brace_token.span,
                "Atleast one variant must be supplied",
            ))
        }
        _ => return Err(Error::new(attr.span(), "Only enums are allowed.")),
    };

    let mut results = vec![];
    for variant in variants {
        let ident = variant.ident;
        let fields = match variant.fields {
            syn::Fields::Named(fields) => fields,
            syn::Fields::Unit => {
                results.push((ident, vec![]));
                continue;
            }
            field => {
                return Err(Error::new(
                    field.span(), //
                    "Only named fields are allowed.",
                ));
            }
        };

        if fields.named.is_empty() {
            return Err(Error::new(
                fields.named.span(), //
                "Named variants must have fields.",
            ));
        }

        results.push((ident, fields.named.into_iter().collect()));
    }

    Ok(results)
}
