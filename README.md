# zippy

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generating is working more or less as it should for range types, higher-order functions (though not closures just yet :]), and tuples.

    $ cat test.z
    fun main (?: 1) : 10 = one_of (id id 5, 6)
    fun id |T| (x: T) = x
    fun one_of |T| (x: T, y: T) = x
    $ zc run test.z
    Error: program quit with code 5
