use heck::ToSnekCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Lit, Meta, parse_macro_input};

#[derive(Debug, Clone)]
enum PrefixConfig {
    /// Use struct name as prefix (default behavior)
    StructName(String),
    /// Use custom prefix
    Custom(String),
    /// No prefix
    None,
}

impl PrefixConfig {
    fn apply_to_field(&self, field_name: &str) -> String {
        match self {
            PrefixConfig::StructName(struct_name) => {
                format!("{}_{}", struct_name, field_name).to_ascii_uppercase()
            }
            PrefixConfig::Custom(prefix) => {
                format!("{}_{}", prefix, field_name).to_ascii_uppercase()
            }
            PrefixConfig::None => field_name.to_ascii_uppercase(),
        }
    }
}

/// Derive macro for EnvConfig trait
///
/// By default, maps struct field names to STRUCT_NAME_FIELD_NAME in UPPER_SNAKE_CASE environment variables.
///
/// Supports struct-level attributes:
/// - `#[env_config(no_prefix)]` - disable prefix, use field names directly
/// - `#[env_config(prefix = "PREFIX")]` - use custom prefix instead of struct name
///
/// Supports field-level attributes:
/// - `#[env_config(skip)]` - skip this field (won't load from env) (must implement Default)
/// - `#[env_config(env = "VAR_NAME")]` - specify custom env var name
/// - `#[env_config(default = "value")]` - specify default value  
/// - `#[env_config(optional)]` - make field optional (must be Option<T>)
/// - `#[env_config(parse_with = "function_name")]` - use custom parser function (takes String, returns T)
/// - `#[env_config(nested)]` - treat field as nested EnvConfig struct (calls T::from_env())
#[proc_macro_derive(EnvConfig, attributes(env_config))]
pub fn derive_env_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse struct-level attributes for prefix configuration
    let prefix_config = parse_struct_prefix_config(&input);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("EnvConfig can only be derived for structs with named fields"),
        },
        _ => panic!("EnvConfig can only be derived for structs"),
    };

    let field_assignments = fields
        .iter()
        .map(|field| generate_field_assignment(field, &prefix_config));

    let expanded = quote! {
        impl ::env_config::EnvConfig for #name {
            type Error = ::env_config::EnvConfigError;

            fn from_env() -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#field_assignments,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_struct_prefix_config(input: &DeriveInput) -> PrefixConfig {
    let struct_name = input.ident.to_string();

    // Convert PascalCase struct name to snake_case for the prefix
    let snake_case_struct_name = struct_name.to_snek_case();

    // Default behavior: use struct name as prefix
    let mut prefix_config = PrefixConfig::StructName(snake_case_struct_name);

    // Check for struct-level attributes
    for attr in &input.attrs {
        if attr.path().is_ident("env_config") {
            if let Meta::List(meta_list) = &attr.meta {
                let nested_result = meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                );

                if let Ok(nested_metas) = nested_result {
                    for nested in nested_metas {
                        match nested {
                            Meta::Path(path) if path.is_ident("no_prefix") => {
                                prefix_config = PrefixConfig::None;
                            }
                            Meta::NameValue(name_value) if name_value.path.is_ident("prefix") => {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: Lit::Str(lit_str),
                                    ..
                                }) = &name_value.value
                                {
                                    prefix_config = PrefixConfig::Custom(lit_str.value());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    prefix_config
}

fn generate_field_assignment(
    field: &Field,
    prefix_config: &PrefixConfig,
) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();

    // Parse attributes
    let mut env_name = prefix_config.apply_to_field(&field_name_str);
    let mut default_expr: Option<syn::Expr> = None;
    let mut is_optional = false;
    let mut skip = false;
    let mut parse_with: Option<syn::Expr> = None;
    let mut is_nested = false;

    for attr in &field.attrs {
        if attr.path().is_ident("env_config") {
            if let Meta::List(meta_list) = &attr.meta {
                let nested_result = meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                );

                if let Ok(nested_metas) = nested_result {
                    for nested in nested_metas {
                        match nested {
                            Meta::Path(path) if path.is_ident("skip") => {
                                skip = true;
                            }
                            Meta::Path(path) if path.is_ident("nested") => {
                                is_nested = true;
                            }
                            Meta::NameValue(name_value) if name_value.path.is_ident("env") => {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: Lit::Str(lit_str),
                                    ..
                                }) = &name_value.value
                                {
                                    env_name = lit_str.value();
                                }
                            }
                            Meta::NameValue(name_value) if name_value.path.is_ident("default") => {
                                // Store the entire expression, not just the string value
                                default_expr = Some(name_value.value.clone());
                            }
                            Meta::NameValue(name_value)
                                if name_value.path.is_ident("parse_with") =>
                            {
                                // Store the parser function expression
                                parse_with = Some(name_value.value.clone());
                            }
                            Meta::Path(path) if path.is_ident("optional") => {
                                is_optional = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Validate attribute combinations
    if skip && (default_expr.is_some() || is_optional || parse_with.is_some() || is_nested) {
        panic!("Cannot use 'skip' with other attributes");
    }

    if is_nested && (default_expr.is_some() || parse_with.is_some()) {
        panic!("Cannot use 'nested' with 'default' or 'parse_with' attributes");
    }

    if parse_with.is_some() && default_expr.is_some() {
        panic!("Cannot use both 'parse_with' and 'default' attributes on the same field");
    }

    // Handle skipped fields
    if skip {
        return quote! {
            #field_name: Default::default()
        };
    }

    // Handle nested EnvConfig structs
    if is_nested {
        let field_type = &field.ty;
        return quote! {
            #field_name: #field_type::from_env()
                .map_err(|e| ::env_config::EnvConfigError::Parse(
                    format!("nested {}", stringify!(#field_type)),
                    e.to_string()
                ))?
        };
    }

    // Handle fields with custom parser
    if let Some(parser_fn) = parse_with {
        // Convert string literal to function identifier
        let parser_ident = if let syn::Expr::Lit(syn::ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) = &parser_fn
        {
            let fn_name = lit_str.value();
            syn::Ident::new(&fn_name, lit_str.span())
        } else {
            // If it's not a string literal, assume it's already a valid expression
            return quote! {
                compile_error!("parse_with must be a string literal containing the function name")
            };
        };

        return if is_optional {
            quote! {
                #field_name: ::env_config::env_var_optional_with_parser(#env_name, #parser_ident)?
            }
        } else {
            quote! {
                #field_name: ::env_config::env_var_with_parser(#env_name, #parser_ident)?
            }
        };
    }

    // Generate the appropriate call based on attributes (existing logic)
    match (default_expr, is_optional) {
        (Some(default), false) => {
            quote! {
                #field_name: ::env_config::env_var_or_parse(#env_name, #default)?
            }
        }
        (None, true) => {
            quote! {
                #field_name: ::env_config::env_var_optional(#env_name)?
            }
        }
        (None, false) => {
            quote! {
                #field_name: ::env_config::env_var(#env_name)?
            }
        }
        (Some(_), true) => {
            panic!("Cannot use both 'default' and 'optional' attributes on the same field")
        }
    }
}
