# Command-line arguments

All commands also take the following options:

- `--no-eval` - skip partial evaluation
- `--output-ir` - output a textual representation of the intermediate
  representation of the code in the artifacts folder
- `--preserve-output` - never overwrite lines in the compiler output
- `--target <target>` - the target to build for (note: currently poorly
  supported)

## `zc check <file>`

Compile and check the given file, but don't produce an output object.

## `zc build <file>`

Compile, check and build the given into an object.

## `zc run <file>`

Build and run the given file.
