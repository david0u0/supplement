use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Result, Token, parse_macro_input};

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
    if ident == "@ext" {
        return "External".to_owned();
    }
    format!("Val{}", to_pascal_case(ident))
}
fn to_cmd_mod(ident: &str) -> String {
    to_snake_case(ident)
}

struct IdList {
    root_mod: syn::Path,
    items: Vec<String>,
}

impl Parse for IdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let root_mod: syn::Path = input.parse()?;

        let mut items = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![@]) {
                input.parse::<Token![@]>()?;
                let ident: syn::Ident = input.parse()?;
                let ident = ident.to_string();
                if ident != "ext" {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("Unknown ident @{ident}"),
                    ));
                }
                if !input.is_empty() {
                    return Err(Error::new(
                        Span::call_site(),
                        "@ext must be the last element",
                    ));
                }
                items.push("@ext".to_owned());
            } else {
                let ident: syn::Ident = input.parse()?;
                items.push(ident.to_string());
            }
        }

        if items.len() < 1 {
            return Err(Error::new(
                Span::call_site(),
                "The macro requires at least two elements",
            ));
        }

        Ok(IdList { items, root_mod })
    }
}

/// Helper macro to simplify the nested ID hell.
///
/// `id!(def remote set_url url)` expands to
/// `def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl)`
///
/// To specify an external subcommand, use `@ext`. E.g. `id!(def parent_cmd @ext)`.
/// This expands to `def::ID::CMDParentCmd(def::parent_cmd::ID::External)`
/// ```rust
/// use supplement_proc_macro::id;
///
/// mod def {
///     #[derive(PartialEq, Eq, Debug)]
///     pub enum ID {
///         ValGitDir,
///         CMDRemote(remote::ID),
///     }
///
///     pub mod remote {
///         #[derive(PartialEq, Eq, Debug)]
///         pub enum ID {
///             CMDSetUrl(set_url::ID),
///             External,
///         }
///
///         pub mod set_url {
///             #[derive(PartialEq, Eq, Debug)]
///             pub enum ID {
///                 ValUrl,
///             }
///         }
///     }
/// }
///
/// // Root flag or arg
/// let e = id!(def git_dir);
/// assert_eq!(e, def::ID::ValGitDir);
///
/// // Flag or arg within subcommand
/// let e = id!(def remote set_url url);
/// assert_eq!(
///     e,
///     def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl))
/// );
///
/// // External subcommands
/// let e = id!(def remote @ext);
/// assert_eq!(e, def::ID::CMDRemote(def::remote::ID::External));
///
/// // Start with a different module def::remote
/// let e = id!(def::remote set_url url);
/// assert_eq!(e, def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl));
///
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    let IdList { items, root_mod } = parse_macro_input!(input as IdList);
    let tokens = build_recur(&root_mod, &items, 0);
    tokens.into()
}

fn build_recur(root_mod: &syn::Path, items: &[String], index: usize) -> proc_macro2::TokenStream {
    let mod_path: Vec<Ident> = items[..index]
        .iter()
        .map(|m| Ident::new(&to_cmd_mod(&*m), Span::call_site()))
        .collect();

    if index == items.len() - 1 {
        let val = Ident::new(&to_val_enum(&items[index]), Span::call_site());
        return quote! {
            #root_mod::#(#mod_path::)*ID::#val
        };
    }

    let cmd = Ident::new(&to_cmd_enum(&items[index]), Span::call_site());
    let inner = build_recur(root_mod, items, index + 1);
    quote! {
        #root_mod::#(#mod_path::)*ID::#cmd(#inner)
    }
}
