# corollary

a smol functional/imperative programming language. currently, the big selling
points are statically typed effects (& effect handlers) with partial evaluation.
also modular implicits, potentially.

as of now, a very basic core works. lexing through typing through code
generating is working more or less as it should for range types, higher-order functions (though not closures just yet :]), and tuples.

on a windows machine in a batch shell (with `link.exe` in path, somehow):

    corollary> type test.z
    -- in test.z
    let main: 0 upto 1 -> ? =
      ? => f (f (f (f id))) 5

    let f: (0 upto 10 -> 0 upto 10) -> ? =
      f => f

    let id = f (x => x)

    corollary> cargo run -q -- test.z
    <snip>

    corollary> link /entry:_WinMain artifacts\test.lib Kernel32.lib /out:artifacts\test.exe
    <snip>

    corollary> artifacts\test.exe

    corollary> echo %ERRORLEVEL%
    5

    corollary> link /dump /disasm artifacts\test.exe
    Microsoft (R) COFF/PE Dumper Version 14.33.31630.0
    Copyright (C) Microsoft Corporation.  All rights reserved.


    Dump of file artifacts\test.exe

    File Type: EXECUTABLE IMAGE

      0000000140001000: E8 08 00 00 00     call        000000014000100D
      0000000140001005: 48 89 F9           mov         rcx,rdi
      0000000140001008: E8 10 00 00 00     call        000000014000101D
      000000014000100D: 55                 push        rbp
      000000014000100E: 48 89 E5           mov         rbp,rsp
      0000000140001011: 48 BF 05 00 00 00  mov         rdi,5
                        00 00 00 00
      000000014000101B: C9                 leave
      000000014000101C: C3                 ret
      000000014000101D: FF 25 DD 0F 00 00  jmp         qword ptr [0000000140002000h]

      Summary

            1000 .rdata
            1000 .text
