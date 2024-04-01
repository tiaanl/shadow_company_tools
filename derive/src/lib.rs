use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[allow(dead_code)]
#[derive(Debug)]
enum ConfigType<'a> {
    Type(&'a syn::Ident),
    Array(&'a syn::Ident),
    Vec(&'a syn::Ident),
}

fn ident_from_type(ty: &syn::Type) -> Option<&syn::Ident> {
    match ty {
        syn::Type::Path(ref path) => path.path.get_ident(),
        _ => None,
    }
}

impl<'a> From<&'a syn::Type> for ConfigType<'a> {
    fn from(value: &'a syn::Type) -> Self {
        if let Some(ident) = ident_from_type(value) {
            return ConfigType::Type(ident);
        }

        if let syn::Type::Array(array) = value {
            if let Some(ident) = ident_from_type(array.elem.as_ref()) {
                return ConfigType::Array(ident);
            }
        }

        if let syn::Type::Path(path) = value {
            if path.path.segments.len() == 1 {
                let first = path.path.segments.first().unwrap();
                if first.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(ref args) = first.arguments {
                        if args.args.len() == 1 {
                            if let syn::GenericArgument::Type(ty) = args.args.first().unwrap() {
                                if let Some(ident) = ident_from_type(ty) {
                                    return ConfigType::Vec(ident);
                                }
                            }
                        }
                    }
                }
            }
        }

        todo!()
    }
}

#[proc_macro_derive(Config, attributes(config))]
pub fn parse_line(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let class_name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        panic!("Deriving Config only allowed on structs.");
    };

    #[derive(Debug)]
    struct ValueParser<'a> {
        field_name: syn::Ident,
        key_name: String,
        config_type: ConfigType<'a>,
    }

    let value_parsers = data
        .fields
        .iter()
        .filter_map(|field| {
            let config_type = ConfigType::from(&field.ty);
            let mut key_name = None;

            field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("config"))
                .for_each(|attr| {
                    let name: syn::LitStr = attr.parse_args().unwrap();
                    key_name = Some(name.value());
                });

            key_name.map(|key_name| ValueParser {
                field_name: field.ident.clone().unwrap(),
                key_name,
                config_type,
            })
        })
        .collect::<Vec<_>>();

    let setters = value_parsers
        .into_iter()
        .map(|parser| {
            let field_name = parser.field_name;
            let key = parser.key_name;
            match parser.config_type {
                ConfigType::Type(_) | ConfigType::Array(_) => quote!(#key => {
                    self.#field_name = ConfigValue::from(line).0;
                }),
                ConfigType::Vec(_) => quote!(#key => {
                    self.#field_name.push(ConfigValue::from(line).0);
                }),
            }
        })
        .collect::<Vec<_>>();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // use shadow_company_tools::config::ConfigLineParser;
        impl #class_name {
            fn parse_config_line(&mut self, line: &ConfigLine) -> bool {
                // println!("Parsing line!");

                match line.name.as_str() {
                    #(#setters)*

                    _ => return false,
                }

                true
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
