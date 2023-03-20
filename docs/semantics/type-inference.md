# Type inference

The type system has two fundamental concepts, the **template** and the **type**.

A type is something describing the shape of some data. A template is a type as
it is attached to some item. Items may introduce locally scoped types in the
form of type parameters or other implicits, but those parameters are not
themselves part of a type. The basic reason can be summarized as a simple rule:

> Every value has a type of kind `type`.

What this means is that, say, a generic type is, unless fully applied, never the
type of a value. Every expression in a program has a monomorphic type - a type
of kind `type`.

To facilitate generic programming, items are allowed to have implicit parameters
including type parameters. These parameters may introduce types (in a similar
fashion to a generic type), but they are *not* part of the type of the item
itself. Instead, they are part of the template. A reference (such as a name) to
an item within an expression is not a reference to that template, but a
particular **instantiation** of that template. It is not possible to refer to
the template, and not an instantiation, which means that all items must have
all their implicit parameters spelled out, even if they are simple renamings of
other items.

An exception to this rule is made for imports. Because it would be cumbersome to
have to specify the implicit parameters of the item referred to by an import,
and because imports may refer to either type items or value items, they are
required to *not* specify implicit parameters.

> To be explicit about the distinction of types and templates, all type
> annotations will, by convention, write the implicit parameters *before* the
> colon. So notating type of an `id` function should look like this:
>
>     id |T| : T -> T
>
> and not like this:
>
>     id : |T| T -> T
>
> This can be read as "`id`, when applied to some type `T`, has the type
> `T -> T`".

## Instantiation

An instantiation of a template is some type where all implicit parameters are
replaced with some concrete type. Note that this process is not "recursive";
templates inside a trait inside another template are not instantiated when the
outer template is instantiated, though they will be instantiated if they are
ever used.

## Templates inside types

Note that a trait, which is a kind of type, contains item declarations, which in
turn define templates of items. This ultimately means that types can *contain*
templates, even if a template can never be directly "held":

    type Id = trait
      fun id |T| (value: T) : T
    
    let normal_id: Id = entry
      fun id |T| value = value
    
    let confused_id: Id = entry
      fun id |T| value = id value

    fun use_it(id: Id) : Bool * String =
      id.id true, id.id "hi!"

## Equating and coercing types

At certain points, implicit coercions are allowed to happen. An implicit
coercion is a "widening" of a type; taking some value of a narrower type and
making it into a value of some type that accepts "more" values.

For instance, a value `5: 0 upto 10` can be coerced to the type `-100 upto 100`,
because the latter includes all the values of the former (and then some). The
reverse is not allowed.

Coercions only happen within expressions; specifically, during the subsumption
check.

> The type checker operates in two different "modes" which recursively call
> each other: a *checking* mode, where an expression is checked against a type,
> and an *inferring* mode, where the type of an expression is inferred.
>
> For instance, to infer the type of `root 5`, we start by inferring the type
> `root: Num -> Num`. Then we check `5` against the type `Num`, and the type of
> the entire expression is `Num`.
>
> Only some expressions (such as numeric literals or lambdas) can be checked
> against a type. For any other expression, we infer the type of the expression
> and coerce that type to the type we are checking against. This is the
> subsumption check, and it is the only place where coercions can happen.

Type equality is determined syntactically. A nominal type is never equal to its
definition, although a value of that type can be coerced to the nominal type
(though not the other way around).

## Partial information

A program is allowed to omit many type annotations, as long as the "missing"
annotations can be recovered by syntactic unification during type equality
checks.

For example, the following program is allowed, since the missing types are
"eventually" resolved:

    let x = 5       -- x: ?1
    let y: Int = x  -- ?1 = Int

But the following is not allowed, since integer literals do not have a default
type, leaving an unsolved unification variable:

    let x = 5  -- x: ?1
               -- ?1 = ???

## Alpha-equality and -coercibility

To properly type check entries and traits, a slightly stronger form of type
equality is required. In particular, look at this code:

    type Id = trait
      fun id |T| (value: T) : T

    let confused: Id = entry
      fun id |U| value = id value

Type inference should give `confused.id |U| : U -> U`. Even though `confused`
itself never `mentions` the type parameter `U` outside its declaration, we can
tell that it is really the same parameter as the `T` in `Id.id`, just with a
different name.

When unifying two traits, it is *alpha-equivalence* which is important here. Two
templates are alpha-equivalent if they have the same number and kind of implicit
parameters, and that those parameters are "used" in the same way. Fortunately,
since implicit parameters must, in this case, always be explicit, this is easy:
just "line them up", check that they are of the same shape, and then unify the
template types using that assumption (of course, some care needs to be taken to
ensure that the resulting types use the correct names, but it is doable).

Coercing two traits should also use a similar notion of *alpha-coercibility*,
but note that the template types need only be coercible, not unifiable.
