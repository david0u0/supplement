use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Pat, Result, Token, parenthesized, parse_macro_input, token::Paren};

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
    items: Vec<(Item, Option<Pat>)>,
}
impl Parse for IdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let root_mod: syn::Path = input.parse()?;
        let mut ctx: Option<Pat> = None;

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
                items.push((Item::Ext, ctx.take()));
            } else if input.peek(Paren) {
                if ctx.is_some() {
                    return Err(Error::new(Span::call_site(), "Ctx cannot be consecutive"));
                }

                let content;
                parenthesized!(content in input);
                ctx = Some(Pat::parse_single(&content)?);
            } else {
                let ident: syn::Ident = input.parse()?;
                items.push((Item::Ident(ident), ctx.take()));
            }
        }

        if items.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "The macro requires at least two elements",
            ));
        }
        if ctx.is_some() {
            return Err(Error::new(Span::call_site(), "Ctx mustn't be the last one"));
        }

        let ext_pos = items.iter().position(|i| matches!(i.0, Item::Ext));
        if ext_pos.is_some() && ext_pos != Some(items.len() - 1) {
            return Err(Error::new(Span::call_site(), "@ext must be the last one"));
        }

        Ok(IdList { items, root_mod })
    }
}

/// Helper macro to simplify the nested ID hell.
///
/// `id!(def remote set_url url)` expands to
/// `def::ID::CMDRemote(_, def::remote::ID::CMDSetUrl(_, def::remote::set_url::ID::ValUrl(_))`
///
/// To specify an external subcommand, use `@ext`. E.g. `id!(def parent_cmd @ext)`.
/// This expands to `def::ID::CMDParentCmd(_, def::parent_cmd::ID::External(_))`
/// ```rust
/// use supplement_proc_macro::id;
///
/// mod def {
///     #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///     pub enum ID {
///         ValGitDir(u32),
///         CMDRemote(u32, remote::ID),
///     }
///
///     pub mod remote {
///         #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///         pub enum ID {
///             External(u32),
///             CMDSetUrl(u32, set_url::ID),
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
///     // To get the context, add `(ctx)` in the macro
///     id!(def(ctx) git_dir) if ctx == 1 => (),
///     // Or to ignore the context
///     id!(def git_dir) => (),
///     _ => panic!(),
/// }
///
/// // Flag or arg within subcommand
/// let e = def::ID::CMDRemote(1, def::remote::ID::CMDSetUrl(2, def::remote::set_url::ID::ValUrl(3)));
/// match e {
///     id!(def(1) remote(2) set_url(3) url) => (),
///     // Or to ignore some (but not all) context
///     id!(def remote set_url(3) url) => (),
///     _ => panic!(),
/// }
///
/// // External subcommands
/// let e = def::ID::CMDRemote(1, def::remote::ID::External(2));
/// match e {
///     id!(def(1) remote(2) @ext) => (),
///     _ => panic!(),
/// }
///
/// // Start with different module def::remote
/// let e = def::remote::ID::CMDSetUrl(2, def::remote::set_url::ID::ValUrl(3));
/// match e {
///     id!(def::remote(2) set_url(3) url) => (),
///     _ => panic!(),
/// }
///
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    let IdList { items, root_mod } = parse_macro_input!(input as IdList);
    let tokens = build_recur(&root_mod, &items, 0);
    tokens.into()
}

fn build_recur(
    root_mod: &syn::Path,
    items: &[(Item, Option<Pat>)],
    index: usize,
) -> proc_macro2::TokenStream {
    let mod_path: Vec<_> = items[..index].iter().map(|i| to_cmd_mod(&i.0)).collect();

    let (item, ctx) = &items[index];
    if index == items.len() - 1 {
        let (item, ctx) = &items[index];
        let val = Ident::new(&to_val_enum(item), Span::call_site());
        let last = quote! {
            #root_mod::#(#mod_path::)*ID::#val
        };
        return if let Some(ctx) = ctx {
            quote! {
                #last(#ctx)
            }
        } else {
            quote! {
                #last(_)
            }
        };
    }

    let cmd = Ident::new(&to_cmd_enum(item), Span::call_site());
    let inner = build_recur(root_mod, items, index + 1);
    if let Some(ctx) = ctx {
        quote! {
            #root_mod::#(#mod_path::)*ID::#cmd(#ctx, #inner)
        }
    } else {
        quote! {
            #root_mod::#(#mod_path::)*ID::#cmd(_, #inner)
        }
    }
}
