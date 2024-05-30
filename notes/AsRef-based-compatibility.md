# `AsRef` base compatibility

Unfortunately blanket impl of compatibility based on presence of `AsRef` in current state of being is not feasible due to missing `AsRef<[T; N]>` implementation for `[T; N]` and specification not being stabilized.

```rust
impl<GL, GLSL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for [GL; N]
where
    GL: ScalarType,
    GLSL: super::Type<Primitive=GL>,
{
    const CHECK_SAME_SIZE: () = assert!(<[GL; N] as glsl::FFI>::SIZE == <glsl::Array<GLSL, N> as glsl::FFI>::SIZE);
    fn as_pod(&self) -> &[GLSL::Primitive] {
        unsafe { std::slice::from_raw_parts(self as *const _, GLSL::SIZE) }
    }
}
```

## Workarounds

1. Ditch using arrays as data

Rather extreme and quite lame since user would need to get into using some whack-ass crate.

2. Force them to wrap arrays into some type which would provide `AsRef`

Ugly, produces boilerplate.

3. Provide `Compatible` impls using macro rather than 

4. Expose unsafe `Compatible` 


## Decision

I'm going with 3. and 4. as it seems right and can be in the future extended to `#[derive]`.

