# About unsafe traits

We need to decive what to do with unsafe traits, do we use them? When do we use them? What's the MEANING of unsafe trait.

It's true that erronous implementation of for example `Target` can cause varying degrees of damage but is this unsafe as per rust's understanding? It's a shame (not really) that coloring of functions are not first class (thank god) this would allow us to make *suggestions* much more convincing than by using docs. Ofc types are preferable here but at point of traits there is really nothing we can do I guess (we can run buildscript which reads values of constants XDDDD).

At the moment we're going to do without `unsafe trait` and we'll see where exploration takes us.

## `unsafe impl` in macro

No macros for unsafe impls. They hide important information which should be explicit.
