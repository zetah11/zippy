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

```asm
main:
.b20:   push rbp
        mov rbp, rsp
        mov rax, 5
        leave
        ret
```

only a simple return remains :)

with partial evaluation disabled (`COR_NO_EVAL`), the following is instead
generated:

```asm
f4:
.b17:   push rbp
        mov rbp, rsp
        leave
        ret

id:
.b18:   push rbp
        mov rbp, rsp
        mov rsi, rdi
        mov rdi, .b4
        call apply
.b19:   leave
        ret

main:
.b20:   push rbp
        mov rbp, rsp
        mov rsi, id
        mov rdi, 5
        call apply
.b21:   leave
        ret

apply:
.b22:   push rbp
        mov rbp, rsp
        mov rdx, rdi
        mov rdi, rsi
        call rdx
.b23:   leave
        ret
```
