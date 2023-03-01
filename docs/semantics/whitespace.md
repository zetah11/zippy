# Whitespace sensitivity

Roughly, the language is designed to map indents to opening parentheses, dedents
to closing parentheses, and newlines to semicolons. For parsing purposes,
however, an indent is a separate token from a parenthesis etc.

> Specifically, separating the two kinds of tokens makes it easier to detect a
> spurious `)` without jumbling up the parser state and producing weird trees.
> It should make no semantic difference in correct programs, however.

In particular, indentation is very "dumb" and works like this:

- The lexer keeps track of a "stack" of indentation levels, starting at the
  leftmost column 0.
- If there is a decrease in the indentation level, `Dedent` tokens are emitted
  until the top of the indentation stack is less than or equal to the current
  indentation level.
- If the indentation level has decreased or stayed the same, an `Eol` token is
  emitted *after* every `Dedent` token.
- If the indentation level has increased from the top of the indentation stack
  (as it was *before* being modified by the `Dedent` pass), a single `Indent`
  token is emitted. Note that this means that a `Dedent` token is never directly
  followed by an `Indent` token.
- When the end of the file is reached, `Dedent` tokens are produced for every
  indentation level on the stack (except 0).
