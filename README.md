# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generating is working more or less as it should for range types, higher-order functions (though not closures just yet :]), and tuples.

in a powershell terminal:

    corollary $ cat .\test.z
    -- in test.z
    fun main (?: 1) = x
    fun id |T| (x: T) = x
    let x: 10 = id 5
    corollary $ cargo r -q -- test.z
    corollary $ .\artifacts\test.exe
    corollary $ echo $LASTEXITCODE  
    5
