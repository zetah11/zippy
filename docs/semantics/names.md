# Names

Broadly, there are two main constructs affecting names: declarative regions and
sequential regions.

A declarative region is some bunch of code where *items* can be freely declared
in any order. An item is some binding, like a `let` binding or `type` binding.
Imports are allowed in declarative regions.

In a sequential region, expressions may only refer to names defined "before"
itself, mimicking the scoping rules in statement-oriented languages.
An expression is always within a sequential region.

Declarative and sequential regions may be arbitrarily nested. Sequential regions
can be introduced with expressions, such as in the body of a `let` binding,
while declarative regions can be introduced with an `entry`.

## Shadowing

Sequential regions follow a strict ordering, so shadowing is allowed. This means
that two sequential names can use the same written identifier. These may be
internally disambiguated with a "sequence number", which increases every time a
sequential name is introduced in a sequential region.

Because names in declarative regions may refer to each other freely (as well as
themselves), shadowing would lead to ambiguity (or a more complicated type
system) and is therefore disallowed.

These rules ensure that a (non-path) name can be reliably resolved using only
lexical information.

## Name resolution

Generally speaking, resolving a name involves finding every name visible at that
point in the program, and then picking whichever is "closest", meaning the most
nested declarative name or latest sequential name (i.e. the one with the largest
sequence number).

## Modules

Modules participate in name resolution as if they were items declared in the
root scope.
