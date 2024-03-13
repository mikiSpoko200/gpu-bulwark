# Enforcing builder compleation in callbacks

When it comes to type driven builders there's a plathera of approaches to choose from.

## Requirements

1. Chaining syntax
2. Enforcement of build action
3. Separate types

Here's the basic outline

```rust
use std::marker::PhantomData;

pub struct Completed<T>(T);

pub trait Builder {
    type Output;

    fn finish(self) -> Completed<Self::Output>;
}

pub struct InnerBuildState(&'static str);
pub struct InnerBuildOutput(&'static str);

pub struct Stage<const STAGE: usize>;

pub struct InnerBuilder<S>(PhantomData<S>);

impl Default for InnerBuilder<Stage<0>> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S> InnerBuilder<S> {
    fn new() -> Self {
        InnerBuilder(Default::default())
    }
}

impl InnerBuilder<Stage<0>> {
    // Implement this step

    pub fn second(self) -> InnerBuilder<Stage<1>> {
        InnerBuilder::<Stage<1>>::new()
    }
}

impl InnerBuilder<Stage<1>> {
    // Implement this step

    pub fn third(self) -> InnerBuilder<Stage<2>> {
        InnerBuilder::<Stage<2>>::new()
    }
}

impl Builder for InnerBuilder<Stage<2>> {
    type Output = InnerBuildOutput;

    fn finish(self) -> Completed<Self::Output> {
        Completed(InnerBuildOutput("siema"))
    }
}

#[derive(Default)]
pub struct OuterBuilder;


impl OuterBuilder {
    pub fn build_inner(self, f: impl FnOnce(InnerBuildState) -> Completed<InnerBuildOutput>) -> Self {
        let _outcome = f(InnerBuildState("hej"));
        self
    }
}

fn main() {
    let outer_builder = OuterBuilder::default();

    outer_builder.build_inner(
        |_state| {
            InnerBuilder::default()
                .second()
                .third()
                .finish()
        }
    );
}
```

The idea is that callback that's tasked with building an inner resource should be expected to return type that only inner builder upon sucessful consturction can produce.
