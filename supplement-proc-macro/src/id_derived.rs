use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Error, Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Paren,
};

/// A single segment like `Git.git_dir` or `Sub.Remote1.verbose(ctx)`
struct IdSegment {
    type_name: Ident,
    parts: Vec<Ident>,
    ctx: Option<Ident>,
}

impl Parse for IdSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let type_name: Ident = input.parse()?;
        let mut ctx = None;
        input.parse::<Token![.]>()?;

        let mut parts = vec![input.parse::<Ident>()?];
        while input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            parts.push(input.parse::<Ident>()?);
        }

        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let ident: Ident = content.parse()?;
            if !content.is_empty() {
                return Err(Error::new(Span::call_site(), "context must be unique"));
            }
            ctx = Some(ident);
        }

        Ok(IdSegment {
            type_name,
            parts,
            ctx,
        })
    }
}

struct IdInput {
    segments: Vec<IdSegment>,
}

impl Parse for IdInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut segments = Vec::new();

        while !input.is_empty() {
            segments.push(input.parse()?);
        }

        Ok(IdInput { segments })
    }
}

fn build_id_expr(segments: &[IdSegment]) -> TokenStream2 {
    if segments.is_empty() {
        return quote! { compile_error!("macro requires at least one segment") };
    }

    let mut result: Option<TokenStream2> = None;

    for segment in segments.iter().rev() {
        let type_name = &segment.type_name;
        let id_type = quote! { <#type_name as Supplement>::ID };

        // The length prefix is based on the FIRST part only (matching derive macro behavior)
        let first_part = segment.parts[0].to_string();
        let combined: String = segment.parts.iter().map(|p| p.to_string()).collect();
        let variant = format_ident!("X{}X{}", first_part.len(), combined);

        result = Some(match (result, &segment.ctx) {
            (Some(inner), None) => quote! { #id_type::#variant(_, #inner) },
            (Some(inner), Some(ctx)) => quote! { #id_type::#variant(#ctx, #inner) },

            (None, None) => quote! { #id_type::#variant(_, ()) },
            (None, Some(ctx)) => quote! { #id_type::#variant(#ctx, ()) },
        });
    }

    result.unwrap()
}

pub fn id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IdInput);
    let expr = build_id_expr(&input.segments);
    TokenStream::from(expr)
}
