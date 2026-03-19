use proc_macro::TokenStream;

mod derive_supplement;
mod id;
mod id_derived;

#[proc_macro_derive(Supplement, attributes(clap, command))]
pub fn derive_supplement(input: TokenStream) -> TokenStream {
    derive_supplement::derive_supplement(input)
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
///     // To bind to the ID, add `(id)` in the macro
///     id!(def(id) git_dir) => {
///         let _tmp: def::ID = id; // `id` binds to the root ID
///     }
///     // Or to make no binding
///     id!(def git_dir) => panic!(),
///     _ => panic!(),
/// }
///
/// // Flag or arg within subcommand
/// let e = def::ID::CMDRemote(1, def::remote::ID::CMDSetUrl(2, def::remote::set_url::ID::ValUrl(3)));
/// match e {
///     id!(def(id1) remote(id2) set_url(id3) url) => {
///         let _tmp: def::ID = id1; // `id1` binds to the root ID
///         let _tmp: def::remote::ID = id2; // `id2` binds to the inner ID
///         let _tmp: def::remote::set_url::ID = id3; // `id3` binds to the inner most ID
///     }
///     // Or to only bind to some id
///     id!(def remote set_url(id3) url) => panic!(),
///     _ => panic!(),
/// }
///
/// // External subcommands
/// let e = def::ID::CMDRemote(1, def::remote::ID::External(2));
/// match e {
///     id!(def(id1) remote(id2) @ext) => (),
///     _ => panic!(),
/// }
///
/// // Start with different module def::remote
/// let e = def::remote::ID::CMDSetUrl(1, def::remote::set_url::ID::ValUrl(2));
/// match e {
///     id!(def::remote(id1) set_url url) => {
///         let _tmp: def::remote::ID = id1; // `id` binds to `def::remote::ID`
///     }
///     _ => panic!(),
/// }
///
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::id(input)
}

/// id_derived!(Git.git_dir) expands to GitID::X7Xgit_dir(_)
/// i_derivedd!(Git.sub(ctx) Sub.Remote1.verbose) expands to GitID::X3Xsub(ctx, SubID::X14XRemote1verbose(_))
#[proc_macro]
pub fn id_derived(input: TokenStream) -> TokenStream {
    id_derived::id(input)
}
