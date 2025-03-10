# latex-expr-parser

a simple parser for latex expressions.
doesn't handle all use cases in latex


## inspired by a few projects
- https://github.com/typst/typst
- https://github.com/matklad/minipratt/tree/master
- https://github.com/rust-lang/rust-analyzer (the parser section)


## create pkg
```bash
cargo install wasm-pack
wasm-pack build --target bundler --features wasm
```