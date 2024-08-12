
# Naming complex types

`gpu-bulwark` makes heavy use of recursive types which are constructed by the api. Due to these types being produced by non const api calls it is difficult to name these types.

Below we discuss possible solutions to that issue.

# DSL Macro that expand to complex types

# Using closure capture

An object can be contained within closure's capture.

| Pros / Cons | Description |
|-------------|-------------|
| -           | Only single closure can exist that owns the data |
| -           | Data would need to be generic any way |
| ?+          | Trait bounds for generic parameter would me more manageable |

# Using type parameters

Some especially nasty types that cannot be named using DSL macro.
