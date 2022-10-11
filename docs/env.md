# Environment variables

## `PRESERVE_OUTPUT`

During partial evaluation, the compiler will report which bindings it is
evaluating. By default, this is reported on a single line, which is overwritten
to prevent the compiler from overflowing the terminal with largely useless
information.

    note: evaluating 'x'

If `PRESERVE_OUTPUT` is set, such lines will not be overwritten.

    note: evaluating 'apply'
    note: evaluating 'id'
    note: evaluating 'x'
