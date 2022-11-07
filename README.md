# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generating is working more or less as it should for range types, higher-order functions (though not closures just yet :]), and tuples.

on a linux machine:

    corollary> cat test.z
    -- in test.z
    fun main (?: 1) = apply (id, 9)
    fun apply (f: 10 -> 10, x) = f x
    fun id (x: 10) = x

    corollary> cargo run -q -- test.z
    <snip>

    corollary> ld artifacts/test.o -o artifacts/test
    corollary> ./artifacts/test
    corollary [9]> objdump -M intel -d artifacts/test

    artifacts/test:     file format elf64-x86-64


    Disassembly of section .text:

    0000000000401000 <_start>:
      401000:       e8 0c 00 00 00          call   401011 <_nmain>
      401005:       48 b8 3c 00 00 00 00    movabs rax,0x3c
      40100c:       00 00 00
      40100f:       0f 05                   syscall

    0000000000401011 <_nmain>:
      401011:       55                      push   rbp
      401012:       48 89 e5                mov    rbp,rsp
      401015:       40 b7 09                mov    dil,0x9
      401018:       c9                      leave
      401019:       c3                      ret
