use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Pat, Result, Token, parse_macro_input, token::Paren};

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

fn to_cmd_enum(item: &Item) -> String {
    match item {
        Item::Ident(ident) => {
            format!("CMD{}", to_pascal_case(&ident.to_string()))
        }
        _ => unreachable!(),
    }
}
fn to_val_enum(item: &Item) -> String {
    match item {
        Item::Ident(ident) => {
            format!("Val{}", to_pascal_case(&ident.to_string()))
        }
        _ => "External".to_owned(),
    }
}
fn to_cmd_mod(item: &Item) -> &Ident {
    match item {
        Item::Ident(ident) => ident,
        _ => unreachable!(),
    }
}

enum Item {
    Ident(Ident),
    Ext,
}
struct IdList {
    root_mod: syn::Path,
    items: Vec<Item>,
    ctxs: Option<Pat>,
}
impl Parse for IdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let root_mod: syn::Path = input.parse()?;
        let mut ctxs = None;

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
                items.push(Item::Ext);
            } else if input.peek(Paren) {
                ctxs = Some(Pat::parse_single(input)?);

                if !input.is_empty() {
                    return Err(Error::new(Span::call_site(), "Ctx must be the last one"));
                }
            } else {
                let ident: syn::Ident = input.parse()?;
                items.push(Item::Ident(ident));
            }
        }

        if items.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "The macro requires at least two elements",
            ));
        }

        let ext_pos = items.iter().position(|i| matches!(i, Item::Ext));
        if let Some(ext_pos) = ext_pos
            && ext_pos != items.len() - 1
        {
            return Err(Error::new(Span::call_site(), "@ext must be the last one"));
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
///                 ValUrl(u32, u32),
///             }
///         }
///     }
/// }
///
/// // Root flag or arg
/// let e = def::ID::ValGitDir(1);
/// match e {
///     // To get the context, add `(ctx)` in the macro
///     id!(def git_dir(ctx)) if ctx == 1 => (),
///     // Or to ignore the context
///     id!(def git_dir) => (),
///     _ => panic!(),
/// }
///
/// // Flag or arg within subcommand
/// let e = def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl(2, 3)));
/// match e {
///     id!(def remote set_url url(ctx1, ctx2)) if ctx1 == 2 && ctx2 == 3 => (),
///     _ => panic!(),
/// }
///
/// // External subcommands
/// let e = def::ID::CMDRemote(def::remote::ID::External(4));
/// match e {
///     id!(def remote @ext(ctx)) if ctx == 4 => (),
///     id!(def remote @ext) => panic!(),
///     id!(def remote @ext) => panic!(),
///     _ => panic!(),
/// }
///
/// // Start with ctx different module def::remote
/// let e = def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl(5, 6));
/// match e {
///     id!(def::remote set_url url(ctx, _)) if ctx == 5 => (),
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
    items: &[Item],
    index: usize,
    ctxs: Option<Pat>,
) -> proc_macro2::TokenStream {
    let mod_path: Vec<_> = items[..index].iter().map(to_cmd_mod).collect();

    if index == items.len() - 1 {
        let val = Ident::new(&to_val_enum(&items[index]), Span::call_site());
        let last = quote! {
            #root_mod::#(#mod_path::)*ID::#val
        };
        return if let Some(ctxs) = ctxs {
            quote! {
                #last #ctxs
            }
        } else {
            quote! {
                #last(..)
            }
        };
    }

    let cmd = Ident::new(&to_cmd_enum(&items[index]), Span::call_site());
    let inner = build_recur(root_mod, items, index + 1, ctxs);
    quote! {
        #root_mod::#(#mod_path::)*ID::#cmd(#inner)
    }
}
