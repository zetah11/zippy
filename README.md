# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generating is working more or less as it should for range types, higher-order functions (though not closures just yet :]), and tuples.

on a linux machine (or wsl):

    $ cat test.z
    -- in test.z
    let main: 0 upto 1 -> ? =
      ? => f (f (f (f id))) 5

    let f: (0 upto 10 -> 0 upto 10) -> ? =
      f => f

    let id = f (x => x)
    $ cargo run -q -- test.z
    <snip>
    $ ld artifacts/main.o -o artifacts/main
    $ ./artifacts/main
    [5] $ objdump -M intel -d artifacts/main

    artifacts/main:     file format elf64-x86-64


    Disassembly of section .text:

    0000000000401000 <_start>:
    401000:       e8 07 00 00 00          call   40100c <main>
    401005:       b8 3c 00 00 00          mov    eax,0x3c
    40100a:       0f 05                   syscall

    000000000040100c <main>:
    40100c:       55                      push   rbp
    40100d:       48 89 e5                mov    rbp,rsp
    401010:       48 bf 05 00 00 00 00    movabs rdi,0x5
    401017:       00 00 00
    40101a:       c9                      leave
    40101b:       c3                      ret
