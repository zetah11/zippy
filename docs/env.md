# Environment variables

## `COR_NO_EVAL`

If `COR_NO_EVAL` is set, no partial evaluation will be performed.

## `COR_OUTPUT_IR`

If `COR_OUTPUT_IR` is set, the intermediate representations of the code will be
dumped to the artifacts folder.

## `COR_PRESERVE_OUTPUT`

During partial evaluation, the compiler will report which bindings it is
evaluating. By default, this is reported on a single line, which is overwritten
to prevent the compiler from overflowing the terminal with largely useless
information.

    note: evaluating 'x'

If `COR_PRESERVE_OUTPUT` is set, such lines will not be overwritten.

    note: evaluating 'apply'
    note: evaluating 'id'
    note: evaluating 'x'
