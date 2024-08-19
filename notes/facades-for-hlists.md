Currently I utilize traits for HList operations. Usually there's trait for single item as well as collection e.g. `Uniform` and `Uniforms` or `Parameter` and `Parameters`.

These function mostly as 

Additionalt notes: collections perhaps could be abstracted? We'd need abstraction over boundry or perhaps the constraint API would solve this? It smells like teen spirit.

`(H, T): HListOf<constraints::Uniform>` this does not eliviate necessitity of providing impls for both HList cases it however unifies the API (no functionality just more consistancy).

However generally constraint API I think comes with added verbosity since we'd need to implement Fulfiled -- Since type bounds need to be satisfied either way (to create type, and in fulfil impl) -- look into this