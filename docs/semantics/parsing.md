# Parsing & syntax

The parser operates on a token stream and is a two-pass process. The first pass
produces a non-semantic concrete syntax tree (in
`crates/zippy-frontend/src/parse/cst.rs`). The second pass reassociates and
transforms this into a semantic abstract syntax tree.

## Motivation

The syntax consists of several parts: declarations, expressions, patterns, and
types. All of these have significant similarities - a name, for instance, is a
valid expression, pattern, and type. This makes it hard to correctly parse some
things in one pass:

    let f = (x: Int, y) => x + y

When the parser gets to the opening parenthesis, it has to make a choice between
parsing a pattern, a type or an expression. In this case, it should parse a
pattern, but in the general case, this requires unbounded lookahead to figure
out. Alternatively, the parser could backtrack once it realizes it made the
wrong choice, but that makes it much more complicated (and complicated machines
are harder to keep robust).

Instead, the parser first just bags everything - declarations, expressions,
patterns and types - into a simple tree of `Item`s. This type represents the
"union" of these different categories. Then a later pass can look at this rough
tree structure, see that the tree is a lambda, and try to convert the right hand
side to a pattern and the left hand side to an expression (emitting error
messages if it fails).

An advantage of this approach is that it makes certain kinds of neat syntax
very easy to parse, such as operator functions:

    fun a + b = add a b

## Syntax

The syntax presented applies to the *first* pass, so it does not describe what
is and is not allowed in the various parts. It is intended for clarifying what
the parser does.

The syntax is split in two parts, **items** and **expressions**. Items roughly
correspond to declarations and "statements", and expressions roughly correspond
to expressions, patterns and types.

The grammar is given in something resembling
[augmented BNF](https://en.wikipedia.org/wiki/Augmented_Backus%E2%80%93Naur_form)
because I think it looks nice (in short, `a / b` denotes a choice between `a` or
`b`, `[a]` denotes an optional `a`, and `*a` denotes zero or more `a`s).
Capitalized words are tokens.

    items      = [item *(delimit item)]
    delimit    = SEMICOLON / EOL

    item       = import / let / expression

    import     = IMPORT expression
    let        = LET expression [EQUAL expression]

    expression = annotate
    annotate   = invoke [COLON invoke]
    invoke     = [invoke PERIOD] simple

    simple     = NAME / NUMBER / STRING
               / LEFT-PAREN items RIGHT-PAREN
               / INDENT items DEDENT
