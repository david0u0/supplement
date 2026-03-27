use proc_macro::TokenStream;

mod derive_supplement;
mod id_codegen;
mod id_derived;

#[proc_macro_derive(Supplement, attributes(clap, command, arg))]
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
/// use supplement_proc_macro::id_codegen as id;
///
/// mod def {
///     #[derive(PartialEq, Eq, Debug, Clone, Copy)]
///     pub enum ID {
///         ValGitDir(u32),
///         CMDRemote(u32, remote::ID),
///     }
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
///     // To bind to the ID, add `(xxx)` to the macro
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
pub fn id_codegen(input: TokenStream) -> TokenStream {
    id_codegen::id(input)
}

/// Helper macro to simplify the nested ID hell.
///
/// - `id!(GitID.git_dir)` expands to `GitID::X7Xgit_dir(_)`
/// - `id!(GitID.sub(acc) SubID.Remote1.verbose)` expands to `GitID::X3Xsub(acc, SubID::X6XRemote1verbose(_))`
///
/// NOTE: [`id!`] is preferred if `#![feature(more_qualified_paths)]` becomes stable
///
/// ```rust
/// mod def {
///     #[derive(Clone, Copy)]
///     pub struct Acc1;
///     #[derive(Clone, Copy)]
///     pub struct Acc2;
///     #[derive(Clone, Copy)]
///     pub struct Acc3;
///
///     #[derive(Clone, Copy)]
///     pub enum GitID {
///         X7Xgit_dir(Acc1, ()),
///         X3Xsub(Acc1, SubID),
///     }
///     #[derive(Clone, Copy)]
///     pub enum SubID {
///         X6XRemotesub(Acc2, RemoteSubID),
///     }
///     #[derive(Clone, Copy)]
///     pub enum RemoteSubID {
///         X6XSetURLurl(Acc3, ()),
///     }
/// }
///
/// use def::*;
/// use supplement_proc_macro::id_no_assoc as id;
///
/// // Root flag or arg
/// let e = GitID::X7Xgit_dir(Acc1, ());
/// match e {
///     // To bind to the accessor, add `(xxx)` to the macro
///     id!(GitID.git_dir(acc)) => {
///         let _acc: Acc1 = acc;
///     }
///     // Or to make no binding
///     id!(GitID.git_dir) => panic!(),
///     _ => panic!(),
/// }
///
/// let e = GitID::X3Xsub(Acc1, SubID::X6XRemotesub(Acc2, RemoteSubID::X6XSetURLurl(Acc3, ())));
/// match e {
///     id!(GitID.sub(acc1) SubID.Remote.sub(acc2) RemoteSubID.SetURL.url(acc3)) => {
///         let _acc: Acc1 = acc1; // `acc1` binds to the root accessor
///         let _acc: Acc2 = acc2; // `acc2` binds to the inner accessor
///         let _acc: Acc3 = acc3; // `acc3` binds to the inner most accessor
///     }
///     // Or to only bind to some accessor
///     id!(GitID.sub SubID.Remote.sub(acc2) RemoteSubID.SetURL.url) => panic!(),
///     _ => panic!(),
/// }
///
/// // Start with different ID
/// let e = SubID::X6XRemotesub(Acc2, RemoteSubID::X6XSetURLurl(Acc3, ()));
/// match e {
///     id!(SubID.Remote.sub(_acc1) RemoteSubID.SetURL.url) => (),
///     _ => panic!(),
/// }
/// ```
#[proc_macro]
pub fn id_no_assoc(input: TokenStream) -> TokenStream {
    id_derived::id(input, false)
}

/// Helper macro to simplify the nested ID hell.
///
/// - `id!(GitID.git_dir)` expands to `<Git as Supplement>::ID::X7Xgit_dir(_)`
/// - `id!(GitID.sub(acc) SubID.Remote1.verbose)` expands to `<Git as Supplement>::ID::ID::X3Xsub(acc, <SubID as Supplement>::ID::X6XRemote1verbose(_))`
///
/// This may cause compile error without `#![feature(more_qualified_paths)]`. In such case, you can use [`id_no_assoc!`].
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id_derived::id(input, true)
}
