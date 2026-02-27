use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token, parse_macro_input};

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase() // TODO
}

fn to_pascal_case(s: &str) -> String {
    let mut ret = String::new();
    for s in s.split('_') {
        let mut chars = s.chars();
        match chars.next() {
            None => continue,
            Some(first) => {
                ret += &first.to_uppercase().to_string();
                ret += &(chars.collect::<String>());
            }
        }
    }
    ret
}

fn to_cmd_enum(ident: &str) -> String {
    format!("CMD{}", to_pascal_case(ident))
}
fn to_val_enum(ident: &str) -> String {
    format!("Val{}", to_pascal_case(ident))
}
fn to_cmd_mod(ident: &str) -> String {
    to_snake_case(ident)
}

struct IdList {
    items: Vec<String>,
}

impl Parse for IdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            items.push(ident.to_string());

            // allow optional whitespace only (space separated)
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(IdList { items })
    }
}

/// `id!(def remote set_url url)` becomes
/// `def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl)`
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    let IdList { items } = parse_macro_input!(input as IdList);
    let mod_name = items.first().unwrap(); // TODO: error
    let items = &items[1..];
    let mod_ident = Ident::new(mod_name, Span::call_site());
    if items.is_empty() {
        panic!(); // TODO better error?
    }
    let tokens = build_recur(&mod_ident, items, 0);
    tokens.into()
}

fn build_recur(mod_ident: &Ident, items: &[String], index: usize) -> proc_macro2::TokenStream {
    let mod_path: Vec<Ident> = items[..index]
        .iter()
        .map(|m| Ident::new(&to_cmd_mod(&*m), Span::call_site()))
        .collect();

    if index == items.len() - 1 {
        let val = Ident::new(&to_val_enum(&items[index]), Span::call_site());
        return quote! {
            #mod_ident::#(#mod_path::)*ID::#val
        };
    }

    let cmd = Ident::new(&to_cmd_enum(&items[index]), Span::call_site());
    let inner = build_recur(mod_ident, items, index + 1);
    quote! {
        #mod_ident::#(#mod_path::)*ID::#cmd(#inner)
    }
}
