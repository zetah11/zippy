# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

    let apply: (0 upto 10 -> 0 upto 10) * (0 upto 10) -> 0 upto 10 =
        (f, x) = f x
    
    let id: 0 upto 10 -> 0 upto 10 = x => x

    let x = apply (id, 5)
