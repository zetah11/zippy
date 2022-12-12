# zippy

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generation works more or less as it should for (some) nominal types, implicits,
numbers and range types, and higher order functions (though no closures just
yet :]).

    $ cat test.z
    type Unit = 1
    type Small = 0 upto 10

    fun main (?: Unit) : Small = one_of (swap (id id 5, 6))

    fun id |T| (x: T) = x
    fun one_of |T| (x: T, y: T) = y
    fun swap |T, U| (x: T, y: U) = y, x

    $ zc run test.z
    Error: program quit with code 5
