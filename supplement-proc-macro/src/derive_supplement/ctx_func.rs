use super::{extract_inner_type, extract_inner_type_opt};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Type, parse_quote};

fn gen_it_type(target: &Type) -> TokenStream2 {
    quote! {
        std::iter::Map<std::slice::Iter<'_, String>, fn(&String) -> #target>
    }
}

pub enum CtxFunc<'a> {
    Count,
    Single(&'a Type, bool),
    Multi(&'a Type, bool),
}
impl CtxFunc<'_> {
    pub fn new(ty: &Type, is_value_enum: bool) -> CtxFunc<'_> {
        let ty = extract_inner_type(ty, &["Option"]);
        if let Type::Path(type_path) = ty {
            let segment = &type_path.path.segments.last().unwrap();
            if segment.ident == "bool" {
                return CtxFunc::Count;
            }
            if let Some(inner) = extract_inner_type_opt(ty, &["Vec"]) {
                return CtxFunc::Multi(inner, is_value_enum);
            }
        }

        CtxFunc::Single(ty, is_value_enum)
    }

    pub fn generate(&self, name: &Ident, uniq_num: u32) -> TokenStream2 {
        fn map_asref(ty: &Type) -> Option<Type> {
            if let Type::Path(type_path) = ty {
                let ident = &type_path.path.segments.last().unwrap().ident;
                if ident == "String" {
                    return Some(parse_quote! { &str });
                }
                if ident == "PathBuf" {
                    return Some(parse_quote! { &std::path::Path });
                }
            }
            None
        }
        match self {
            CtxFunc::Count => quote! {
                pub fn #name(self, seen: &Seen) -> u32 {
                    seen.find(id::NoVal::new(#uniq_num)).map(|u| u.count).unwrap_or_default()
                }
            },
            CtxFunc::Single(ty, true) => quote! {
                pub fn #name(self, seen: &Seen) -> Option<std::result::Result<#ty, String>> {
                    seen.find(id::SingleVal::new(#uniq_num)).map(|u| <#ty as ValueEnum>::from_str(&u.value, false))
                }
            },
            CtxFunc::Multi(ty, true) => {
                let target = parse_quote! { std::result::Result<#ty, String> };
                let it_type = gen_it_type(&target);
                quote! {
                    pub fn #name(self, seen: &Seen) -> #it_type {
                        let v = seen.find(id::MultiVal::new(#uniq_num)).map(|u| u.values.as_slice()).unwrap_or(&[]);
                        v.iter().map(|s| <#ty as ValueEnum>::from_str(&s, false))
                    }
                }
            }

            CtxFunc::Single(ty, false) => {
                if let Some(as_ref) = map_asref(ty) {
                    quote! {
                        pub fn #name(self, seen: &Seen) -> Option<#as_ref> {
                            seen.find(id::SingleVal::new(#uniq_num)).map(|u| u.value.as_ref())
                        }
                    }
                } else {
                    quote! {
                        pub fn #name(self, seen: &Seen) -> Option<std::result::Result<#ty, <#ty as FromStr>::Err>> {
                            seen.find(id::SingleVal::new(#uniq_num)).map(|u| u.value.parse())
                        }
                    }
                }
            }
            CtxFunc::Multi(ty, false) => {
                if let Some(as_ref) = map_asref(ty) {
                    let it_type = gen_it_type(&as_ref);
                    quote! {
                        pub fn #name(self, seen: &Seen) -> #it_type {
                            let v = seen.find(id::MultiVal::new(#uniq_num)).map(|u| u.values.as_slice()).unwrap_or(&[]);
                            v.iter().map(|s| s.as_ref())
                        }
                    }
                } else {
                    let target = parse_quote! { std::result::Result<#ty, <#ty as FromStr>::Err> };
                    let it_type = gen_it_type(&target);
                    quote! {
                        pub fn #name(self, seen: &Seen) -> #it_type {
                            let v = seen.find(id::MultiVal::new(#uniq_num)).map(|u| u.values.as_slice()).unwrap_or(&[]);
                            v.iter().map(|s| s.parse())
                        }
                    }
                }
            }
        }
    }
}
