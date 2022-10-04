# Potential ideas

## Effects; typed and handled

One-shot (affine) effects with (complete?) effect inference.

    type Throws Ex where
      eff throw: Ex -> Never / Throws
    
    let throw = Throws.throw
    
    let catch: ('ex -> 'a / 'f) -> (Unit -> 'a / 'ex # 'e) -> 'a / 'e # 'f
    let catch handle tried =
      with Throws ? handler
        eff throws ex = handle ex
      tried ()
    
    type DivBy where
      def zero: Int -> DivBy
    
    let div: Int -> Int -> Int
    let div a 0 = throw DivBy.zero
      | div a b = a / b
    
    fun main =
      println
        Int.to_string (div 5 0)
        with catch (DivBy.zero => "division by zero!")

## (Modular) implicits

Implicits resolved through scope closeness + unification (for any type variables
constrained by the implicit).

    type Ordering = enum
      less
      equal
      greater

    type Ordered = trait
      type T
      def compare: T -> T -> Ordering
    
    let (<) @(O: Ordering) : O.T -> O.T -> bool
    let (<) @O a b = O.compare a b = Ordering.less

    let sort @(O: Ordered) (it: List O.T) =
      var swapped = true
      while swapped do
        swapped := false
        for i in 0 upto it.length - 1 do
          if it[i + 1] < it[i] do
            swapped := true
            it[i], it[i + 1] := it[i + 1], it[i]
    
    let sort_rev @(O: Ordered) (it: List O.T) =
      use let rev_ordering: Ordered = class
        type T = O.T
        let compare a b = O.compare b a
      sort it

## Partial evaluation

Partial evaluation of pure code as an easy optimization + const eval-like
functionality.

    let big: SomeNums = 42
    let value: Sixteen = big  -- error: 'SomeNums' wider than 'Sixteen'

    alias Sixteen  = 0 upto 65536
    alias SomeNums = 0 upto fib 25

    fun fib: Nat -> Nat
    fun fib 0 = 0
      | fib 1 = 1
      | fib n = fib (n - 1) + fib (n - 2)

## First-class-ish modules through singleton types

Modules can be typed with a `trait` type, and are simply inhabitants of that
type. For reasons of monomorphism etc., each module value is associated with a
particular singleton type of that trait, such that no actual overhead is
associated with this.

Only `trait`s with associated types need singleton types for their
implementation. Everything else can be considered a record. This way, we don't
end up with any unhandleable/existential effects on function calls.

    type Stack = trait
      type T
      type Element
      let push: var T -> Element -> Unit
      let pop: var T -> Maybe Element
    
    let ListStack: Stack = class
      type T = List Int
      type Element = Int

      let push = List.push
      let pop  = List.pop
    
    let LinkedStack: Stack = class
      type T where
        def nil: T
        def cons: Int -> T -> T
      type Element = Int

      let push (var self) value = self := T.cons value self 
      let pop (var self) =
        case self
          is T.nil => none
          is T.cons head tail =>
            self := tail
            some head
    
    let StackImpl = if something do ListStack else LinkedStack
    -- error: ListStack and LinkedStack have distinct types.

## Heavily overloadable syntax (aka lispification)

Almost any non-literal and non-function-application expression is actually just
function application with special syntax.

    let (:) @T (x: T) = x
    
    let (->): 'a -> 'b -> 'a * 'b
    let a -> b = a, b

    fun (if): Bool -> 'a -> 'a
    fun (if) (true)  then _    = then
      | (if) (false) _    elze = elze

## Eager and lazy, CBPV

Support both eager and lazy bindings, with automatic forcing and delaying
wherever necessary to make it work.

    fun main =
      -- 'let's are eager, 'fun's are lazy
      let x: Int / Pure    = println "hi!"; 5
      fun y: Int / Console = println "hay"; 10
      println "hello"
      println (x + y)
    
    -- Output:
    -- hi!
    -- hello
    -- hay
    -- 15

> Sidenote: this is a much more difficult problem than what it may appear in the
> suggestive presentation above, due to lambdas and constructors:
>
>     fun f =
>       let g = x => x
>       g
