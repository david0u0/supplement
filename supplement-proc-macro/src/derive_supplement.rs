use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Ident, PathArguments, Type,
    parse_macro_input,
};

const ID_POSTFIX: &str = "IDGeneratedBySupplement";
const CTX_POSTFIX: &str = "CtxGeneratedBySupplement";

enum CtxFunc<'a> {
    Count,
    Single(&'a Type),
    Multi(&'a Type),
}
impl CtxFunc<'_> {
    fn generate(&self, name: &Ident) -> TokenStream2 {
        match self {
            CtxFunc::Count => quote! {
                pub fn #name(self) -> u32 {
                    unimplemented!()
                }
            },
            CtxFunc::Single(ty) => quote! {
                pub fn #name(self) -> Option<#ty> {
                    unimplemented!()
                }
            },
            CtxFunc::Multi(ty) => quote! {
                pub fn #name(self) -> impl Iterator<Item = #ty> {
                    vec![].into_iter()
                }
            },
        }
    }
}

fn map_ctx_func(ty: &Type) -> CtxFunc<'_> {
    let ty = extract_inner_type(ty, &["Option"]);
    if let Type::Path(type_path) = ty {
        let segment = &type_path.path.segments.last().unwrap();
        if segment.ident == "bool" {
            return CtxFunc::Count;
        }
        if segment.ident == "Vec" {
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(GenericArgument::Type(inner)) = args.args.first() {
                    return CtxFunc::Multi(inner);
                }
            }
        }
    }

    CtxFunc::Single(ty)
}

fn has_subcommand_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("clap") || attr.path().is_ident("command") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("subcommand") {
                return true;
            }
        }
    }
    false
}

fn extract_inner_type<'a>(ty: &'a Type, outer_types: &[&str]) -> &'a Type {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last().unwrap();
        if outer_types.iter().any(|o| segment.ident == o) {
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(GenericArgument::Type(inner)) = args.args.first() {
                    return inner;
                }
            }
        }
    }
    ty
}

/// (Log, pretty) => X3XLogpretty
/// (git_dir) => X7Xgit_dir
fn format_variant_name(first_name: &str, second_name: Option<&str>) -> syn::Ident {
    let len = first_name.len();
    format_ident!("X{}X{}{}", len, first_name, second_name.unwrap_or_default())
}

fn impl_struct(name: &syn::Ident, fields: &syn::FieldsNamed) -> TokenStream2 {
    let id_name = format_ident!("{name}{ID_POSTFIX}");
    let ctx_name = format_ident!("{name}{CTX_POSTFIX}");

    let mut variants: Vec<TokenStream2> = Vec::new();
    let mut ctx_funcs: Vec<TokenStream2> = Vec::new();
    let mut regular_field_matches: Vec<TokenStream2> = Vec::new();
    let mut subcommand_delegates: Vec<TokenStream2> = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let ctx_func = map_ctx_func(field_ty);
        let field_name_str = field_name.to_string();
        let variant_name = format_variant_name(&field_name_str, None);

        if has_subcommand_attr(&field.attrs) {
            let inner_type = extract_inner_type(&field.ty, &["Option"]);
            variants.push(quote! {
                #variant_name(#ctx_name, <#inner_type as Supplement>::ID)
            });
            subcommand_delegates.push(quote! {
                if let Some(id) = #inner_type::id_from_cmd(cmd) {
                    return Some(Self::ID::#variant_name(Default::default(), id));
                }
            });
        } else {
            variants.push(quote! {
                #variant_name(#ctx_name)
            });
            // TODO: use real CLI name!
            regular_field_matches.push(quote! {
                #field_name_str if cmd.len() == 1 => return Some(Self::ID::#variant_name(Default::default()))
            });
            ctx_funcs.push(ctx_func.generate(field_name));
        }
    }

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct #ctx_name;
        impl #ctx_name {
            #(#ctx_funcs)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #id_name {
            #(#variants),*
        }

        impl Supplement for #name {
            type ID = #id_name;
            fn id_from_cmd(cmd: &[&str]) -> Option<Self::ID> {
                let first = cmd.first()?;

                // Try regular fields first
                match *first {
                    #(#regular_field_matches,)*
                    _ => {}
                }

                // Try subcommand fields
                #(#subcommand_delegates)*

                None
            }
        }
    }
}

fn impl_enum(name: &syn::Ident, data: &syn::DataEnum) -> TokenStream2 {
    let id_name = format_ident!("{name}{ID_POSTFIX}");

    let mut variants: Vec<TokenStream2> = Vec::new();
    let mut ctx_defs: Vec<TokenStream2> = Vec::new();
    let mut from_cmd_arms: Vec<TokenStream2> = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        let variant_name_lower = variant_name_str.to_lowercase();

        match &variant.fields {
            // Named fields: Remote1 { verbose: bool, sub: Remote }
            Fields::Named(fields) => {
                let ctx_name = format_ident!("{name}{CTX_POSTFIX}{variant_name}");
                let mut ctx_funcs: Vec<TokenStream2> = Vec::new();

                let mut field_matches: Vec<TokenStream2> = Vec::new();
                let mut subcommand_delegates: Vec<TokenStream2> = Vec::new();

                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;
                    let ctx_func = map_ctx_func(field_ty);
                    let field_name_str = field_name.to_string();
                    let field_name_lower = field_name_str.to_lowercase();
                    let id_variant_name =
                        format_variant_name(&variant_name_str, Some(&field_name_str));

                    // TODO: possible values

                    if has_subcommand_attr(&field.attrs) {
                        let inner_type = extract_inner_type(field_ty, &["Option"]);
                        variants.push(quote! {
                            #id_variant_name(#ctx_name, <#inner_type as Supplement>::ID)
                        });
                        subcommand_delegates.push(quote! {
                            if let Some(id) = #inner_type::id_from_cmd(rest) {
                                return Some(Self::ID::#id_variant_name(Default::default(), id));
                            }
                        });
                    } else {
                        variants.push(quote! {
                            #id_variant_name(#ctx_name)
                        });
                        field_matches.push(quote! {
                            #field_name_lower if rest.len() == 1 => return Some(Self::ID::#id_variant_name(Default::default()))
                        });

                        ctx_funcs.push(ctx_func.generate(field_name));
                    }
                }

                ctx_defs.push(quote! {
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    pub struct #ctx_name;
                    impl #ctx_name {
                        #(#ctx_funcs)*
                    }
                });

                // TODO: use real CLI name!
                from_cmd_arms.push(quote! {
                    #variant_name_lower => {
                        let rest = &cmd[1..];
                        if rest.is_empty() {
                            return None;
                        }
                        let field_normalized = rest[0].to_lowercase();

                        // Try regular fields
                        match field_normalized.as_str() {
                            #(#field_matches,)*
                            _ => {}
                        }

                        // Try subcommand fields
                        #(#subcommand_delegates)*

                        None
                    }
                });
            }
            // Tuple fields: Remote2(RemoteStruct)
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    variants.push(quote! {
                        compile_error!("Tuple variants must have exactly one field")
                    });
                    continue;
                }

                let field = fields.unnamed.first().unwrap();
                let inner_type = extract_inner_type(&field.ty, &["Option"]);
                let id_variant_name = format_variant_name(&variant_name_str, None);

                variants.push(quote! {
                    #id_variant_name((), <#inner_type as Supplement>::ID)
                });

                // TODO: use real CLI name!
                from_cmd_arms.push(quote! {
                    #variant_name_lower => {
                        let rest = &cmd[1..];
                        if let Some(id) = #inner_type::id_from_cmd(rest) {
                            return Some(Self::ID::#id_variant_name((), id));
                        }
                        None
                    }
                });
            }
            // Unit variant
            Fields::Unit => {
                // No data at all, do nothing
            }
        }
    }

    quote! {
        #(#ctx_defs)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #id_name {
            #(#variants),*
        }

        impl Supplement for #name {
            type ID = #id_name;
            fn id_from_cmd(cmd: &[&str]) -> Option<Self::ID> {
                let first = cmd.first()?;

                match *first {
                    #(#from_cmd_arms,)*
                    _ => None
                }
            }
        }
    }
}

pub fn derive_supplement(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => impl_struct(name, fields),
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Supplement only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(data) => impl_enum(name, data),
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "Supplement cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    TokenStream::from(expanded)
}
