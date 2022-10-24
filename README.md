# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

currently, the following

    let main: 0 upto 1 -> ? =
      ? => apply (id, 5)

    let id = x => apply ((y => y), x)

    let apply: (0 upto 10 -> 0 upto 10) * (0 upto 10) -> 0 upto 10 =
      (f, x) => f x

generates the following x64 machine code

    main: push rbp
          mov rbp, rsp
          mov rax, 5
          leave
          ret

only a simple return remains :)
