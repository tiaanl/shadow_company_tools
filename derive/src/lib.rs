use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[allow(dead_code)]
#[derive(Debug)]
enum ConfigType<'a> {
    String,
    U32,
    Bool,
    UserType(&'a syn::Ident),
    Array(Box<ConfigType<'a>>),
    Vec(Box<ConfigType<'a>>),
}

impl<'a> ConfigType<'a> {
    fn quote(&self, field_name: &syn::Ident) -> proc_macro2::TokenStream {
        match self {
            ConfigType::String | ConfigType::U32 => {
                let setter = self.convert();
                quote!(
                    let __param_index = 0;
                    self.#field_name = #setter;
                )
            }
            ConfigType::Bool => quote!(
                self.#field_name = true;
            ),
            ConfigType::UserType(ident) => quote!(
                self.#field_name = #ident::from(line.clone());
            ),
            ConfigType::Array(element_type) => {
                let convert = element_type.convert();
                quote!(
                    if line.params.len() < self.#field_name.len() {
                        panic!("Not enough parameters to fill array!");
                    }

                    for (__param_index, param) in self.#field_name.iter_mut().enumerate() {
                        *param = #convert;
                    }
                )
            }
            ConfigType::Vec(element_type) => {
                let setter = element_type.convert();
                quote!(
                    self.#field_name.push(#setter);
                )
            }
        }
    }

    fn convert(&self) -> proc_macro2::TokenStream {
        match self {
            ConfigType::String => quote!(line.params[__param_index].clone()),
            ConfigType::U32 => quote!(line.params[__param_index].parse().unwrap()),
            ConfigType::UserType(ident) => quote!(#ident::from(line)),
            _ => unreachable!("no converter for non native types."),
        }
    }
}

fn native_type_from_type(ty: &syn::Type) -> Option<ConfigType> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(ident) = type_path.path.get_ident() {
            if type_path.path.is_ident("String") {
                return Some(ConfigType::String);
            } else if type_path.path.is_ident("u32") {
                return Some(ConfigType::U32);
            } else if type_path.path.is_ident("bool") {
                return Some(ConfigType::Bool);
            } else {
                return Some(ConfigType::UserType(ident));
            }
        }
    }
    None
}

impl<'a> From<&'a syn::Type> for ConfigType<'a> {
    fn from(value: &'a syn::Type) -> Self {
        // Check for native types.
        if let Some(native_type) = native_type_from_type(value) {
            return native_type;
        }

        if let syn::Type::Array(type_array) = value {
            if let Some(native_type) = native_type_from_type(type_array.elem.as_ref()) {
                return ConfigType::Array(Box::new(native_type));
            }
        }

        if let syn::Type::Path(path) = value {
            if path.path.segments.len() == 1 {
                let first = path.path.segments.first().unwrap();
                if first.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(ref args) = first.arguments {
                        if args.args.len() == 1 {
                            if let syn::GenericArgument::Type(ty) = args.args.first().unwrap() {
                                if let Some(native_type) = native_type_from_type(ty) {
                                    return ConfigType::Vec(Box::new(native_type));
                                }
                            }
                        }
                    }
                }
            }
        }

        match value {
            syn::Type::Array(_) => todo!("Array"),
            syn::Type::BareFn(_) => todo!("BareFn"),
            syn::Type::Group(_) => todo!("Group"),
            syn::Type::ImplTrait(_) => todo!("ImplTrait"),
            syn::Type::Infer(_) => todo!("Infer"),
            syn::Type::Macro(_) => todo!("Macro"),
            syn::Type::Never(_) => todo!("Never"),
            syn::Type::Paren(_) => todo!("Paren"),
            syn::Type::Path(_) => todo!("Path"),
            syn::Type::Ptr(_) => todo!("Ptr"),
            syn::Type::Reference(_) => todo!("Reference"),
            syn::Type::Slice(_) => todo!("Slice"),
            syn::Type::TraitObject(_) => todo!("TraitObject"),
            syn::Type::Tuple(_) => todo!("Tuple"),
            syn::Type::Verbatim(_) => todo!("Verbatim"),
            _ => todo!("<other>"),
        }
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
            let setter = parser.config_type.quote(&field_name);
            quote!(#key => {
                #setter;
            })
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
