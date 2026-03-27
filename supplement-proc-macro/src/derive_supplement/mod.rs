use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use std::cell::Cell;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Ident, Lit, Meta,
    PathArguments, Type, parse_macro_input, parse_quote,
};

mod acc_func;
use acc_func::AccFunc;

const ID_NAME: &str = "ID";
const ACC_POSTFIX: &str = "Accessor";
const MOD_POSTFIX: &str = "_mod_generated_by_supplement";

thread_local! {
    static COUNTER: Cell<u32> = const { Cell::new(1) };
}

fn uniq_num() -> u32 {
    let ret = COUNTER.get();
    COUNTER.set(ret + 1);
    ret
}

fn gen_never() -> TokenStream2 {
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Never { }
    }
}

fn has_flat_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("clap") || attr.path().is_ident("command") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("flatten") {
                return true;
            }
        }
    }
    false
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
fn has_externalsubcommand_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("clap") || attr.path().is_ident("command") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("external_subcommand") {
                return true;
            }
        }
    }
    false
}
fn extract_cmd_name(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("clap") && !attr.path().is_ident("command") {
            continue;
        }

        let nested = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .ok()?;

        for meta in nested {
            let Meta::NameValue(nv) = meta else {
                continue;
            };
            if !nv.path.is_ident("name") {
                continue;
            }
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
            {
                return Some(s.value());
            }
        }
    }
    None
}

fn has_value_enum_attr(attrs: &[Attribute]) -> bool {
    // For now, only fields with "value_enum" attribute will have it's ID skipped.
    // Otherwise we'll still generate it and it will be an unreachable arm during completion.
    // TODO: maybe "specialization" can save us?
    // Implement `Supplement` differently on `FromStr` and `ValueEnum`, so that their `ID` is () and Never?
    // I tried some autoref-specialization magic, but it doesn't work for associate type :(
    for attr in attrs {
        if attr.path().is_ident("clap") || attr.path().is_ident("arg") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("value_enum") {
                return true;
            }
        }
    }
    false
}

fn is_bool(ty: &Type) -> bool {
    let ty = extract_inner_type(ty, &["Vec", "Option"]);
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last().unwrap();
        if segment.ident == "bool" {
            return true;
        }
    }
    false
}

fn extract_inner_type_opt<'a>(ty: &'a Type, outer_types: &[&str]) -> Option<&'a Type> {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last().unwrap();
        if outer_types.iter().any(|o| segment.ident == o)
            && let PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            return Some(inner);
        }
    }
    None
}
fn extract_inner_type<'a>(ty: &'a Type, outer_types: &[&str]) -> &'a Type {
    extract_inner_type_opt(ty, outer_types).unwrap_or(ty)
}

/// (Log, pretty) => X3XLogpretty
/// (git_dir) => X7Xgit_dir
fn format_variant_name(first_name: &str, second_name: Option<&str>) -> syn::Ident {
    let len = first_name.len();
    format_ident!("X{}X{}{}", len, first_name, second_name.unwrap_or_default())
}

fn impl_struct(name: &syn::Ident, fields: &syn::FieldsNamed) -> TokenStream2 {
    let id_name = format_ident!("{ID_NAME}");
    let acc_name = format_ident!("{ACC_POSTFIX}");
    let mod_name = format_ident!("{name}{MOD_POSTFIX}");

    let mut variant_names: Vec<Ident> = Vec::new();
    let mut variant_inner_tys: Vec<TokenStream2> = Vec::new();
    let mut acc_funcs: Vec<TokenStream2> = Vec::new();
    let mut acc_fields: Vec<TokenStream2> = Vec::new();
    let mut regular_field_matches: Vec<TokenStream2> = Vec::new();
    let mut subcommand_delegates: Vec<TokenStream2> = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let is_value_enum = has_value_enum_attr(&field.attrs);
        let is_subcommand = has_subcommand_attr(&field.attrs);
        let is_flat = has_flat_attr(&field.attrs);
        let acc_func = AccFunc::new(field_ty, is_value_enum);
        let field_name_str = field_name.to_string();
        let variant_name = format_variant_name(&field_name_str, None);
        variant_names.push(variant_name.clone());

        let uniq_num = uniq_num();
        if is_subcommand || is_flat {
            let inner_type = extract_inner_type(&field.ty, &["Option"]);
            variant_inner_tys.push(quote! { <#inner_type as Supplement>::ID });
            subcommand_delegates.push(quote! {
                if let Some((id, num)) = #inner_type::id_from_cmd(cmd) {
                    let id = id.map(|id| Self::ID::#variant_name(Default::default(), id));
                    return Some((id, num));
                }
            });
            if is_flat {
                acc_fields.push(quote! {
                    pub #field_name: <#inner_type as Supplement>::Accessor,
                });
            }
        } else if is_value_enum || is_bool(field_ty) {
            variant_inner_tys.push(quote! { Never });

            regular_field_matches.push(quote! {
                #field_name_str if cmd.len() == 1 => return Some((None, #uniq_num))
            });
            acc_funcs.push(acc_func.generate(field_name, uniq_num));
        } else {
            variant_inner_tys.push(quote! { () });

            regular_field_matches.push(quote! {
                #field_name_str if cmd.len() == 1 => return {
                    let id = Self::ID::#variant_name(Default::default(), ());
                    Some((Some(id), #uniq_num))
                }
            });
            acc_funcs.push(acc_func.generate(field_name, uniq_num));
        }
    }

    let never = gen_never();
    quote! {
        mod #mod_name {
            use super::*;
            use supplement::{id, Seen};
            use std::str::FromStr;
            use std::ops::Deref;
            use clap::ValueEnum;

            #never

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct #acc_name {
                #(#acc_fields)*
            }
            impl #acc_name {
                #(#acc_funcs)*
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #id_name {
                #(#variant_names(#acc_name, #variant_inner_tys)),*
            }
            impl Deref for #id_name {
                type Target = #acc_name;
                fn deref(&self) -> &Self::Target {
                    // TODO: if we have const Default, we won't need this
                    match self {
                        #(
                            #id_name::#variant_names(acc, ..) => acc,
                        )*
                    }
                }
            }

            impl Supplement for #name {
                type ID = #id_name;
                type Accessor = #acc_name;
                fn id_from_cmd(cmd: &[impl AsRef<str>]) -> Option<(Option<Self::ID>, u32)> {
                    let first = cmd.first()?;

                    // Try regular fields first
                    match first.as_ref() {
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
}

fn pascal_to_dash(s: &str) -> String {
    let mut snake = String::new();
    let mut prev_is_upper = true;
    for ch in s.chars() {
        if ch.is_uppercase() {
            if !prev_is_upper {
                snake.push('-');
            }
            for lc in ch.to_lowercase() {
                snake.push(lc);
            }
            prev_is_upper = true;
        } else {
            prev_is_upper = false;
            snake.push(ch);
        }
    }

    snake
}

fn impl_enum(name: &syn::Ident, data: &syn::DataEnum) -> TokenStream2 {
    let id_name = format_ident!("{ID_NAME}");
    let mod_name = format_ident!("{name}{MOD_POSTFIX}");

    let mut variants: Vec<TokenStream2> = Vec::new();
    let mut acc_defs: Vec<TokenStream2> = Vec::new();
    let mut from_cmd_arms: Vec<TokenStream2> = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        let cmd_name =
            extract_cmd_name(&variant.attrs).unwrap_or_else(|| pascal_to_dash(&variant_name_str));

        match &variant.fields {
            // Named fields: Remote1 { verbose: bool, sub: Remote }
            Fields::Named(fields) => {
                let acc_name = format_ident!("{variant_name}{ACC_POSTFIX}");
                let mut acc_funcs: Vec<TokenStream2> = Vec::new();
                let mut acc_fields: Vec<TokenStream2> = Vec::new();

                let mut field_matches: Vec<TokenStream2> = Vec::new();
                let mut subcommand_delegates: Vec<TokenStream2> = Vec::new();

                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;
                    let is_value_enum = has_value_enum_attr(&field.attrs);
                    let is_subcommand = has_subcommand_attr(&field.attrs);
                    let is_flat = has_flat_attr(&field.attrs);
                    let acc_func = AccFunc::new(field_ty, is_value_enum);
                    let field_name_str = field_name.to_string();
                    let id_variant_name =
                        format_variant_name(&variant_name_str, Some(&field_name_str));
                    let uniq_num = uniq_num();

                    if is_subcommand || is_flat {
                        let inner_type = extract_inner_type(field_ty, &["Option"]);
                        variants.push(quote! {
                            #id_variant_name(#acc_name, <#inner_type as Supplement>::ID)
                        });
                        subcommand_delegates.push(quote! {
                            if let Some((id, num)) = #inner_type::id_from_cmd(rest) {
                                let id = id.map(|id| Self::ID::#id_variant_name(Default::default(), id));
                                return Some((id, num));
                            }
                        });
                        if is_flat {
                            acc_fields.push(quote! {
                                pub #field_name: <#inner_type as Supplement>::Accessor,
                            });
                        }
                    } else if is_value_enum || is_bool(field_ty) {
                        variants.push(quote! {
                            #id_variant_name(#acc_name, Never)
                        });
                        field_matches.push(quote! {
                            #field_name_str if rest.len() == 1 => return Some((None, #uniq_num))
                        });
                        acc_funcs.push(acc_func.generate(field_name, uniq_num));
                    } else {
                        variants.push(quote! {
                            #id_variant_name(#acc_name, ())
                        });
                        field_matches.push(quote! {
                            #field_name_str if rest.len() == 1 => return {
                                let id = Self::ID::#id_variant_name(Default::default(), ());
                                Some((Some(id), #uniq_num))
                            }
                        });

                        acc_funcs.push(acc_func.generate(field_name, uniq_num));
                    }
                }

                acc_defs.push(quote! {
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    pub struct #acc_name {
                        #(#acc_fields)*
                    }
                    impl #acc_name {
                        #(#acc_funcs)*
                    }
                });

                // TODO: use real CLI name!
                from_cmd_arms.push(quote! {
                    #cmd_name => {
                        let rest = &cmd[1..];
                        if rest.is_empty() {
                            return None;
                        }
                        let first = rest[0].as_ref();

                        // Try regular fields
                        match first {
                            #(#field_matches,)*
                            _ => {}
                        }

                        // Try subcommand fields
                        #(#subcommand_delegates)*

                        None
                    }
                });
            }
            // Tuple fields: Remote2(RemoteStruct) or external subcommand
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    variants.push(quote! {
                        compile_error!("Tuple variants must have exactly one field")
                    });
                    continue;
                }
                let id_variant_name = format_variant_name(&variant_name_str, None);
                let field = fields.unnamed.first().unwrap();

                if has_externalsubcommand_attr(&variant.attrs) {
                    let uniq_num = uniq_num();
                    let acc_name = format_ident!("{variant_name}{ACC_POSTFIX}");
                    let acc_func = AccFunc::new(&field.ty, false);
                    variants.push(quote! { #id_variant_name(#acc_name, ()) });
                    from_cmd_arms.push(quote! {
                        // NOTE: "" is for external subcommand
                        "" => {
                            let id = Self::ID::#id_variant_name(Default::default(), ());
                            Some((Some(id), #uniq_num))
                        }
                    });

                    let func_name: Ident = parse_quote! { values };
                    let func = acc_func.generate(&func_name, uniq_num);
                    acc_defs.push(quote! {
                        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                        pub struct #acc_name;
                        impl #acc_name {
                            #func
                        }
                    });
                } else {
                    let inner_type = extract_inner_type(&field.ty, &["Option"]);

                    variants.push(quote! {
                        #id_variant_name((), <#inner_type as Supplement>::ID)
                    });

                    // TODO: use real CLI name!
                    from_cmd_arms.push(quote! {
                        #cmd_name => {
                            let rest = &cmd[1..];
                            if let Some((id, num)) = #inner_type::id_from_cmd(rest) {
                                let id = id.map(|id| Self::ID::#id_variant_name(Default::default(), id));
                                return Some((id, num));
                            }
                            None
                        }
                    });
                }
            }
            // Unit variant
            Fields::Unit => {
                // No data at all, do nothing
            }
        }
    }

    let never = gen_never();
    quote! {
        mod #mod_name {
            use super::*;
            use supplement::{id, Seen};
            use std::str::FromStr;
            use std::ops::Deref;
            use clap::ValueEnum;

            #never
            #(#acc_defs)*

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #id_name {
                #(#variants),*
            }
            impl Deref for #id_name {
                type Target = ();
                fn deref(&self) -> &Self::Target {
                    &()
                }
            }

            impl Supplement for #name {
                type ID = #id_name;
                type Accessor = ();
                fn id_from_cmd(cmd: &[impl AsRef<str>]) -> Option<(Option<Self::ID>, u32)> {
                    let first = cmd.first()?;

                    match first.as_ref() {
                        #(#from_cmd_arms,)*
                        _ => None
                    }
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
