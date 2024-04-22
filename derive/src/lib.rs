use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug)]
enum ConfigType<'a> {
    String,
    U32,
    I32,
    F32,
    Bool,
    UserType(&'a syn::Ident),
    #[allow(dead_code)]
    Array(Box<ConfigType<'a>>),
    Vec(Box<ConfigType<'a>>),
}

impl<'a> ConfigType<'a> {
    fn quote(
        &self,
        field_name: &syn::Ident,
        end_name: Option<&String>,
    ) -> proc_macro2::TokenStream {
        match self {
            ConfigType::String | ConfigType::U32 | ConfigType::I32 | ConfigType::F32 => {
                quote! {
                    self.#field_name = line.param(0).unwrap_or_default();
                    reader.next_line()?;
                }
            }
            ConfigType::Bool => quote!(
                self.#field_name = true;
                reader.next_line()?;
            ),
            ConfigType::UserType(ident) => {
                let end_part = end_name
                    .map(|e| quote! { Some(#e) })
                    .unwrap_or(quote! { None });

                quote! {
                    self.#field_name = #ident::from(line.clone());
                    reader.next_line()?;
                    self.#field_name.parse_config(reader, #end_part)?;
                }
            }
            ConfigType::Array(_) => {
                quote!(
                    if line.params.len() < self.#field_name.len() {
                        panic!("Not enough parameters to fill array!");
                    }

                    for (__param_index, param) in self.#field_name.iter_mut().enumerate() {
                        *param = line.param(__param_index).unwrap_or_default();
                    }
                    reader.next_line()?;
                )
            }
            ConfigType::Vec(element_type) => {
                let setter = element_type.convert();
                let end_part = end_name
                    .map(|e| quote! { Some(#e) })
                    .unwrap_or(quote! { None });

                quote! {
                    self.#field_name.push(#setter);
                    reader.next_line()?;
                    self.#field_name.last_mut().unwrap().parse_config(reader, #end_part)?;
                }
            }
        }
    }

    fn convert(&self) -> proc_macro2::TokenStream {
        match self {
            ConfigType::String => quote!(line.param(__param_index).unwrap_or_default()),
            ConfigType::U32 => quote!(line.param(__param_index).unwrap_or_default()),
            ConfigType::I32 => quote!(line.param(__param_index).unwrap_or_default()),
            ConfigType::F32 => quote!(line.param(__param_index).unwrap_or_default()),
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
            } else if type_path.path.is_ident("i32") {
                return Some(ConfigType::I32);
            } else if type_path.path.is_ident("f32") {
                return Some(ConfigType::F32);
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

#[derive(Debug)]
struct Field<'a> {
    field_name: syn::Ident,
    key_name: Option<String>,
    end_name: Option<String>,
    param_index: Option<usize>,
    config_type: ConfigType<'a>,
}

fn get_assignment_value(assign: &syn::ExprAssign) -> Result<(syn::Path, String), syn::Error> {
    let syn::Expr::Path(path) = assign.left.as_ref() else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Identifier expected on the left of the assignment.",
        ));
    };

    let syn::Expr::Lit(lit) = assign.right.as_ref() else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Only string literals allowed on the right side of the assignment.",
        ));
    };

    let syn::Lit::Str(ref str) = lit.lit else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Only string literals allowed on the right side of the assignment.",
        ));
    };

    Ok((path.path.clone(), str.value()))
}

fn build_fields(fields: &syn::Fields) -> Result<Vec<Field>, syn::Error> {
    let mut result = vec![];

    for field in fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("config") {
                let mut key_name = None;
                let mut end_name = None;

                let nested = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                )?;
                for nest in nested {
                    match nest {
                        syn::Expr::Assign(ref assign) => {
                            //
                            let (ident, value) = get_assignment_value(assign)?;
                            if ident.is_ident("key") {
                                key_name = Some(value);
                            } else if ident.is_ident("end") {
                                end_name = Some(value);
                            } else {
                                return Err(syn::Error::new(
                                    proc_macro2::Span::call_site(),
                                    "Only key and end allowed as values.",
                                ));
                            }
                        }
                        syn::Expr::Lit(lit) => match lit.lit {
                            syn::Lit::Str(str) => key_name = Some(str.value()),
                            _ => {
                                return Err(syn::Error::new(
                                    proc_macro2::Span::call_site(),
                                    "Only string literals allowed for key name.",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new(
                                proc_macro2::Span::call_site(),
                                "Invalid parameter",
                            ))
                        }
                    }
                }

                result.push(Field {
                    field_name: field.ident.clone().unwrap(),
                    key_name,
                    end_name,
                    param_index: None,
                    config_type: ConfigType::from(&field.ty),
                });
            } else if attr.path().is_ident("param") {
                let param_index: syn::LitInt = attr.parse_args()?;

                result.push(Field {
                    field_name: field.ident.clone().unwrap(),
                    key_name: None,
                    end_name: None,
                    param_index: Some(param_index.base10_parse::<usize>().map_err(|_| {
                        syn::Error::new(proc_macro2::Span::call_site(), "Invalid parameter index.")
                    })?),
                    config_type: (&field.ty).into(),
                });
            }
        }
    }

    Ok(result)
}

#[proc_macro_derive(Config, attributes(config, param))]
pub fn parse_line(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        panic!("Deriving Config only allowed on structs.");
    };

    let fields = build_fields(&data.fields).expect("failed to build fields");

    let setters = fields
        .iter()
        .filter(|field| field.key_name.is_some())
        .map(|field| {
            let key = field.key_name.clone().unwrap();
            let setter = field
                .config_type
                .quote(&field.field_name, field.end_name.as_ref());
            quote!(#key => {
                #setter;
            })
        })
        .collect::<Vec<_>>();

    let mut from_config_line_setters = vec![];

    for field in fields.iter() {
        if let Some(param_index) = field.param_index {
            let field_name = &field.field_name;

            let setter = match field.config_type {
                ConfigType::Vec(ref _element) => {
                    // TODO: The type of element should be checked for an appropriate setter.
                    quote! {
                        #field_name: line.params.iter()
                            .skip(#param_index)
                            .flat_map(|e| shadow_company_tools::config::FromParam::from(e.clone()))
                            .collect(),
                    }
                }
                _ => quote! {
                    #field_name: line.param(#param_index).unwrap_or_default(),
                },
            };

            from_config_line_setters.push(setter);
        }
    }

    let and_default = if data.fields.len() != from_config_line_setters.len() {
        quote! {
            ..Default::default()
        }
    } else {
        quote!()
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #struct_name {
            fn parse_config_line<R>(
                &mut self,
                reader: &mut shadow_company_tools::config::ConfigReader<R>,
            ) -> std::io::Result<bool>
            where
                R: std::io::Read + std::io::Seek,
            {
                debug_assert!(reader.current().is_some());

                let line = reader.current().unwrap();

                // println!("{}: Parsing: {}", stringify!(#struct_name), line.name);

                match line.name.as_str() {
                    #(#setters)*

                    _ => {
                        // println!("{}: unknown line name: {}", stringify!(#struct_name), line.name);
                        return Ok(false)
                    },
                }

                #[allow(unreachable_code)]
                Ok(true)
            }

            pub fn parse_config<R>(
                &mut self,
                reader: &mut shadow_company_tools::config::ConfigReader<R>,
                end: Option<&str>,
            ) -> std::io::Result<()>
            where
                R: std::io::Read + std::io::Seek,
            {
                loop {
                    if let (Some(line), Some(end)) = (reader.current(), end) {
                        if line.name == end {
                            reader.next_line()?;
                            break;
                        }
                    }

                    if reader.current().is_none() {
                        break;
                    }

                    if !self.parse_config_line(reader)? {
                        break;
                    }
                }

                Ok(())
            }
        }

        impl From<&shadow_company_tools::config::ConfigLine> for #struct_name
        where
            Self: Default
        {
            fn from(line: &shadow_company_tools::config::ConfigLine) -> Self {
                Self {
                    #(#from_config_line_setters)*
                    #and_default
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
