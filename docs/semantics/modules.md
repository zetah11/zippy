# Modules

A module is a logical grouping of some sources under a "namespace". Each module
may define items, and those items are children of that module. Modules may refer
to items defined in other modules either with a path or by using `import` to
create a source-local alias of that item.

Modules are themselves items and can therefore be children of other modules,
forming a hierarchy.

Modules may be split over several sources or *parts*. Each item in a part may
refer to items defined in the same module (even from other parts) or any parent
modules.

## Imports

Imports are a source-level construct. An import item in one source only
introduces the imported aliases in that source. All other sources are
unaffected, including those which are a part of the same module.

An import consists of some expression and a list of names.

    -- in Another.Module
    import Some.Module.(name1; name2)

The names are made available for the source to use, but they are not part of the
declared items of the module. For example, another module may not refer to
`Another.Module.name1` above.

Imports may also rename the imported items, using `as`:

    import Some.Module.(name1 as first_name; name2 as second_name)

The expression can be any expression, and is not required to refer to a module
(there is very little semantic distinction between modules and objects).

    import (object (let x: 10 = 5)).x

## Name resolution

In order to resolve the names used within a module, we need to compute

- Every item declared by that module
- Every item declared by any of its ancestor modules

Any name not found among these names must be either part of some path or
declared an import. Such names create *aliases*, which are resolved to an actual
name at a later stage.
