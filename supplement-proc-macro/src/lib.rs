use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Result, Token, parenthesized, parse_macro_input, token::Paren};

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
    ctxs: Vec<String>,
}

impl Parse for IdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let root_mod: syn::Path = input.parse()?;
        let mut ctxs = vec![];

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
                items.push("@ext".to_owned());
            } else if input.peek(Paren) {
                let content;
                parenthesized!(content in input);

                if content.is_empty() {
                    return Err(Error::new(Span::call_site(), "Ctx can't be empty"));
                }

                let content = content.parse_terminated(Ident::parse, Token![,])?;
                ctxs = content.iter().map(|ident| ident.to_string()).collect();

                if !input.is_empty() {
                    return Err(Error::new(Span::call_site(), "Ctx must be the last one"));
                }
            } else {
                let ident: syn::Ident = input.parse()?;
                items.push(ident.to_string());
            }
        }

        if items.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "The macro requires at least two elements",
            ));
        }

        Ok(IdList {
            items,
            root_mod,
            ctxs,
        })
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
///     #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///     pub enum ID {
///         ValGitDir(u32),
///         CMDRemote(remote::ID),
///     }
///
///     pub mod remote {
///         #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///         pub enum ID {
///             CMDSetUrl(set_url::ID),
///             External(u32),
///         }
///
///         pub mod set_url {
///             #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///             pub enum ID {
///                 ValUrl(u32),
///             }
///         }
///     }
/// }
///
/// // Root flag or arg
/// let e = def::ID::ValGitDir(1);
/// match e {
///     // To get the ctx, add `(ctx)` in the macro
///     id!(def git_dir(ctx)) if ctx == 1 => (),
///     // Or to ignore the ctx
///     id!(def git_dir) => (),
///     _ => panic!(),
/// }
///
/// // Flag or arg within subcommand
/// let e = def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl(2)));
/// match e {
///     id!(def remote set_url url(ctx)) if ctx == 2 => (),
///     _ => panic!(),
/// }
///
/// // External subcommands
/// let e = def::ID::CMDRemote(def::remote::ID::External(3));
/// match e {
///     id!(def remote @ext(ctx)) if ctx == 3 => (),
///     id!(def remote @ext) => panic!(),
///     _ => panic!(),
/// }
///
/// // Start with ctx different module def::remote
/// let e = def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl(4));
/// match e {
///     id!(def::remote set_url url(ctx)) if ctx == 4 => (),
///     _ => panic!(),
/// }
///
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    let IdList {
        items,
        root_mod,
        ctxs,
    } = parse_macro_input!(input as IdList);
    let tokens = build_recur(&root_mod, &items, 0, ctxs);
    tokens.into()
}

fn build_recur(
    root_mod: &syn::Path,
    items: &[String],
    index: usize,
    ctxs: Vec<String>,
) -> proc_macro2::TokenStream {
    let mod_path: Vec<_> = items[..index]
        .iter()
        .map(|m| Ident::new(&to_cmd_mod(m), Span::call_site()))
        .collect();

    if index == items.len() - 1 {
        let val = Ident::new(&to_val_enum(&items[index]), Span::call_site());
        let last = quote! {
            #root_mod::#(#mod_path::)*ID::#val
        };
        return if ctxs.is_empty() {
            quote! {
                #last(..)
            }
        } else {
            let ctxs: Vec<_> = ctxs
                .iter()
                .map(|m| Ident::new(m, Span::call_site()))
                .collect();
            quote! {
                #last(#(#ctxs, )*)
            }
        };
    }

    let cmd = Ident::new(&to_cmd_enum(&items[index]), Span::call_site());
    let inner = build_recur(root_mod, items, index + 1, ctxs);
    quote! {
        #root_mod::#(#mod_path::)*ID::#cmd(#inner)
    }
}
