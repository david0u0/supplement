# supplements\_example

This example will demonstrate the steps to use `supplements` in your CLI app:

1. Have a `clap` definition
    - [src/args.rs](src/args.rs)
2. Generate the `supplements` definition (preferably in `build.rs`)
    - [build.rd](build.rs) and [Cargo.toml](Cargo.toml)
3. Import and use the generated code
    - [srd/main.rd](src/main.rs)
4. Compile the binary
    - [install.sh](install.sh)
5. Put a simple shell script in place to tell the shell how to use your binary
    - [install.sh](install.sh)

## The clap definition
In [src/args.rs](src/args.rs) you can find a small clap definition which mimics our beloved version control tool `git`. Let's call our toy app `qit`. It contains two subcommand: `checkout` and `log`, and some arguments and flags.

Here I list some not-so-trivial stuff in the definition, and they're all supported by `supplements`.
- `--gir-dir` is global
- `files` in `checkout` can be infinitely long
- `--color` and `--pretty` in `log` have *possible values*. For example, `color` should be one of `auto` `always` or `never`
- `--pretty` in `log` has `num_args = 0..=1, default_value = "short, default_missing_value = "full", require_equals = true`... which just means it behaves differently with or without the equal sign
    + `qit log` is the same as `qit log --pretty=short`
    + `qit log --pretty` is the same as `qit log --pretty=full`
    + `qit log --pretty full` will not supply the argument `full` to `--pretty`

## Generate the `supplements` definition
In theory you can call the generate function anywhere you like. In this example we call it in [build.rd](build.rs), which generates a file `$OUT_DIR/definition.rs`.

NOTE that you should also add the dependency in the `build-dependencies` section of [Cargo.toml](Cargo.toml), because `build.rs` doesn't use the normal dependencies.
```toml
[build-dependencies]
supplements = "0.1"
```

The generated file should look like something along the line of `target/debug/build/supplements-example-15e87e4a170fb654/out/definition.rs`

## Import and use the generated code
In [src/main.rs](src/main.rs), import the generated file as a module. The program has three modes:
- **Completion mode** - `cargo run -- [zsh/bash/fish] qit ...`. Run the completion function for specific shell.
    + Note that the whitespace is significant: `cargo run -- fish check` means `qit check<TAB>`, while `cargo run -- fish check ''` means `qit check <TAB>`
- **Generate mode** - `cargo run -- generate`. Generate the supplements definition and print to stdout.
- **Parse mode** - `cargo run -- [anything else]`. Parse the comment line argument into clap object and print it.

Obviously the `Completion mode` is our main focus, so let's look into it deeper.

```rust
let args = args[2..].iter().map(String::from);
let (history, grp) = def::CMD.supplement(args).unwrap();
let ready = match grp {
    CompletionGroup::Ready(r) => {
        // The easy path. No custom logic needed.
        // e.g. Completing a subcommand or flag, like `git chec<TAB>`
        // or completing something with candidate values, like `ls --color=<TAB>`
        r
    }
    CompletionGroup::Unready { unready, id, value } => {
        let comps = handle_comp(history, id, &value);
        unready.to_ready(comps)
    }
};
ready.print(shell, &mut stdout()).unwrap();
```

The function `def::CMD.supplement` returns a `Result<(History, CompletionGroup)>`.
`History` records everything seen while parsing the command, which your completion may later depends on.
`CompletionGroup` is a enum that has two variant: `Ready` and `Unready`

### Ready
`Ready` is The easy path. No custom logic needed. You should just call `Ready::print` and end the process.

This can happen when e.g. Completing a subcommand or flag, like `git chec<TAB>`, or completing something with candidate values, like `ls --color=<TAB>`.

### Unready
`Unready` is the hard path. Your custom logic should go here. You'll have the history, the id of last seen element (flag or argument), and the value in commandline (possibly `''`).

For example, if you do `cargo run -- fish checkout file1 file2 ''`, the history will contain `file1` and `file2`, the id will be `ID::Checkout(def::checkout::ID::File)`, and the value will be `''`.

You have to convert it to `Ready` object to print it, hence the `to_ready` function, whose input is a vector of `Completion`.
The vector should be computed by *YOU* based on the history, id, value, and anything else you're intrested in.

In [src/main.rs](src/main.rs) I wrote a function `handle_comp` to collect these custom logic.
For example, the `--gir-dir` should simply be complete with files, and `checkout <FILES>` should be completed with modified files in git status.

### Ready::print
The final step. Tell it which shell you're using and fire!

## Install
If you like, you can actually install this toy app `qit` to your system along with it's completion.

The [install.sh](install.sh) will ask a `usr` path from you, and try to copy the binary there, along with the completion script for `zsh`, `bash` and `fish`.

After that, you can open a new shell and play with it.

```sh
qit <TAB> # checkout log
qit -<TAB> # --git-dir
qit checkout <TAB> # some commit hash & files
```

The shell completion scripts can be found in [shell](shell).
They're tiny because the goal of `supplements` is to let you write as little shell as possible.
So it should be easy to tweak them to fit your need -- but don't overdo it and defeat the purpose of `supplements` 😞
