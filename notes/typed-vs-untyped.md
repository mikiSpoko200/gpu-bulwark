# Type of not to type

Type driven APIs allow us to statically enforce certain invariants. This sentence really does say it all, including keyword *statically* on which I'll focus in this post.

Static verification requires amongst other things that you know exact types you use at compile time. This make creating software which operates on different types of data like editors, renderers, web servers, basically anything which is not just a pure computation somewhat tricky.

## Example using buffers

I've been working on software implementation of rasterization pipeline for the *computer graphics and numerical analysis seminar*. Following in the steps of gpu-bulwark I've begun strictly typing most parameters of common objects like buffers or textures. This however made allocation API, well, strongly typed and not very fun to work with.

This lead me to creating Enums with variant per each acceptable buffer type -- basically undoing all the hard work put into typing everything in the first place.

## Well, looks like this *string typing* was a little too much *string typing* for you, Ken.

Typing, especially abuse of the type-checker with polymorphically recursive types can be a bit discouraging for beginners.
For that reason, typing version light should become a thing. There are numerous way of going about this.

## Polymorphism with enums

In case of gpipe I went for enum based polymorphism which basically means enum variant for each distinct type. Actual operations can be delegated easily enough with a little macro magic going on.

```rust
struct Vec<T, const N: usize>(PhantomData<T>);

type Vec4 = Vec<f32, 4>;
type Vec3 = Vec<f32, 3>;
type Vec2 = Vec<f32, 2>;

// Ignore colliding item names
pub enum Vec {
    Vec2(Vec2), // it would be nice to create impls for enum variants like impl Vec::Vec2
    Vec3(Vec3),
    Vec4(Vec4),
}
```

Alternatively we could structure types as `T<A>` use `A` would contain typing information dough it makes design much less idiomatic? (type aliases could have).
Then `()` could mean type erased -- this calls back to subtyping. -- this will be obismal.

Vector<>
