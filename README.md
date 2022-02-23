# iclu

> icelk command line utilities (or icelk core light utilities) are a collection of small CLIs I've built to make the terminal life easier.

Iclu are a set of command line utilities to fill in the gaps of the command line. I regularly use all of these to simplify common tasks.

# Examples

The `ran` and `byc` are used in the [pass](`pass.sh`) script.

For examples of the `corpl`, see [my theme change script](https://github.com/Icelk/dotfiles/blob/main/scripts/theme-change.sh) and [my Polybar configuration](https://github.com/Icelk/dotfiles/blob/main/config/polybar.ini)

# Features

- ran - cryptographic random number generator
- byc - byte conversion, takes integers and turns them into Unicode characters
- corpl - smart commenter, can comment and uncomment scripts/config files to implement light-weight setting sets, such as themes
- shc - shell convert, turns (basic) Unix scripts into Windows Batch scripts
- spl - split input and joint output, while using streams for best performance

If you want another feature (or binary), open an issue and I'll make it if it sounds like a good idea.

You can of course also write the _Rust_ code and open a PR.

# Contributing

The source is available under the Apache 2.0 license, and all contributions should also be.
