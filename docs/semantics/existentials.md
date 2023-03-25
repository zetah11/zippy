# Existentials

A trait type is an existential type, which means it hides some of its
information in a way that makes it look slightly more general than it is.

Specifically, traits are existential over their implementation. Thus, any
actually concrete trait type - one with every existential fully instantiated -
is a (sort of) singleton type.

> TODO: Entries with identical abstract types may be able to have the same
> concrete trait type. This gives the ability to be run-time polymorphic over
> existential types, without losing some of the effect safety and code
> generation guarantees.

## Existential inference

Concretely, traits take an existential parameter (denoted here by `'a`, `'b`,
etc). This is not possible to notate in the surface syntax, and only exists in
typed IRs.

Existentials are inferred on a whole-program basis, following a system inspired
by the Hindley-Milner type system. Existential inference should not affect the
inferred types in a program. That is, the type of an expression will not change
even if the inferred existentials change. Existential inference is, however,
allowed to reject some programs which are otherwise type correct.

Every `entry` in a program generates a unique "existential instance". This
is an opaque thing representing the "origin" of a trait. For example,

    let inc = entry
      fun call(value: Nat) = value + 1

becomes something like the following:

    instance _inc =
      fun call(value: Nat) = value + 1

    let inc = entry _inc

The "desugared" entry is more like a function taking such an instance as an
argument.

`trait`s also take instances as arguments. Thus, the type for `inc` above is a
trait like

    trait _inc
      fun call(value: Nat) : Nat

Two traits are equivalent only if they are alpha-equivalent and have the same
existential instance. Thus, if there was another instance `_dec`, the traits
`trait _inc (...)` and `trait _inc (...)` would be equivalent, while
`trait _dec (...)` would not be equivalent with either.

Likewise, one trait is coercible to another if they are alpha-coercible and have
the same existential instance, because there is no notion of coercion with
existential instances.

If a type is used without any constraints on its existential arguments, then
those arguments will generalize (similar to generalization in HM). This
generalization only happens for existentials in the body of a type definition,
or those which appear in the type of a value definition. Thus, if a function
has an unconstrained existential argument due to some shifty things happening
in its body, but that argument does not appear in the type of the function
itself, then an ambiguity error is raised.

> TODO: Do some deeper analysis on the cases where this might happen (if it is
> even possible) and if ambiguity errors are necessary.

## Example

As a full example, consider a "generalized" Fibonacci function, which applies
a closure to its result before returning:

    type Fun T U = trait
      fun call(arg: T) : U
    
    fun gen_fib (f: Fun Nat Nat) (n: Nat) : Nat =
      if n < 2 do
        n
      else
        f.call(gen_fib f (n - 1) + gen_fib (n - 2))
    
    let inc: Fun Nat Nat = entry
      fun call value = value + 1
    
    let result = gen_fib inc 15

Annotating the existentials, using `_ident` for the names of existential
instances and `'a`, `'b`, etc., for the names of existential arguments, gives
something like

    type Fun 'a T U = trait 'a
      fun call(arg: T) : U
    
    fun gen_fib 'a (f: Fun 'a Nat Nat) (n: Nat) : Nat =
      if n < 2 do
        n
      else
        f.call(gen_fib 'a f (n - 1) + gen_fib 'a f (n - 2))
    
    instance _inc =
      fun call(value: Nat) : Nat = value + 1
    
    let inc: Fun _inc Nat Nat = entry _inc

    let result: Nat = gen_fib _inc inc 15

## Further work

### Returning an entry

Consider a function like this:

    fun Person name' age' = entry
      let name : String = name'
      let age  : Nat    = age'

      fun introduce() =
        print("Hi, I'm ", name, "!")
    
    let jane = Person "Jane Doe" 50
    let john = Person "John Doe" 75

This function is essentially a "constructor" in the OOP sense. Since the
function returns an entry, its type should be something like

    Person : String -> Nat -> trait _person (...)

for some concrete instance `_person`. And because it is concrete, `jane` and
`john` should have equivalent types. However, the instance `_person` is a
little bit tricky, since it contains items which refer to local variables
(namely the parameters of `Person`).

What's worse, it might be desirable to allow something like

    fun bounded (upper: Nat) = entry
      type T = 0 upto upper
      let add (x: T) (y: T) =
        (x as Nat) + (y as Nat)

Now a local appears inside a type (*almost* dependently). How should this be
reckoned with, ideally while still allowing functions to return *different*
entries which have the same concrete type.

One solution may be for instances to have *dependencies*, referring to every
local variable the entry depends on. As such, the `Person` example might become
something like

    instance _person depends Person.name', Person.age' =
      let name : String = name'
      let age  : Nat    = age'

      fun introduce() =
        print("Hi, I'm ", name, "!")
    
    fun Person (name': String) (age': Nat) : trait Person (...) =
      entry _person with name', age'

    let jane: trait _person (...) = Person "Jane Doe" 50
    let john: trait _person (...) = Person "John Doe" 50

Dependencies become a bag of run-time values which values of that type must then
carry with them (i.e. types with instances with dependencies become closures).
However, dependencies don't appear in the trait type, so values with different
dependencies may still have the same type.

> This is okay because for that to happen, the two values must have the same
> instances, which also mean they must have the same types for the dependencies.
> So even if the dependency *values* are only known at runtime, their type (and
> size) is completely statically resolvable.

### Records vs entries/traits

Existentals sort of enforce a "monomorphisation" restriction on existential
types like traits. This is great, since it makes modules and closures zero-cost
while keeping the language relatively expressive, but it does enforce a
significant restriction in the sense that no two entries may ever share the same
type (unless one is literally a copy of the other). How would one do, say, a
database with that?

A record could be a trait-like type without
