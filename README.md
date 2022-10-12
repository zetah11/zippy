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

generates the following ir

    let apply =
      fun _t11 _t0 =
        let f = _t0.0
        let x = _t0.1
        let _t1 = f x
        return _t1
      return _t11

    let id =
      fun _t12 x =
        return x
      return _t12

    let main =
      fun _t13 _t7 =
        return 5
      return _t13

note that `id` has been optimized to a standard `id`, and that `main` references
neither
