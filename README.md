# latex-expr-parser
A simple parser for latex expressions into full AST trees with operation precedence.

I really tried finding a latex parser (in rust) that actually goes all the way down to individual elements with operation precendece, but didn't find
any out there. If you know of some parser that does this please message me!
Every other parser I found doesn't actually parse all the way down to each operation and leaves some of them as just "String" in the AST.
I tried a couple:
1. https://github.com/alongwy/unlatex
2. https://github.com/typst/typst - not really latex but was close enough for me
3. katex
Which makes sense because most of them are made for rendering latex and not evaluating the expression.

I am not trying to parse all latex expressions. I only parsed enough for another project, so I am pretty sure this is
not very useful, but it was a fun exercise though :) 

This whole thing is based on this [great blog](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) by @matklad.

Hopefully this can help you a little with understanding partt parsers.

## inspired by a few projects
- https://github.com/typst/typst
- https://github.com/matklad/minipratt/tree/master
- https://github.com/rust-lang/rust-analyzer (the parser section)


## create pkg
```bash
cargo install wasm-pack
    wasm-pack build --target bundler --features wasm
```

## Testing
I use cargo-insta for testing so to test
```bash
cargo test
cargo insta review - to update the snapshots
```