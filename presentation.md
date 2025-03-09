---
title: "Analysis of Type-Driven approach to systems programming: Implementation of OpenGL library for Rust"
author: "Mikołaj Depta"
date: "03.09.2024"
theme: "simple"
revealOptions:
    transition: "none"
    width: 1600
    height: 1000
    margin: 0.04
    minScale: 0.2
    maxScale: 0.8
---


<style>
    .reveal .slides {
        text-align: left;
    }
    .reveal .slides .image {
        text-align: center;
    }
    
    .container{
        display: flex;
    }
    .col{
        flex: 1;
    }
</style>

*Analysis of Type-Driven approach to systems programming: Implementation of OpenGL library for Rust*

<br/>

Author: Mikołaj Depta

Supervisor: dr. Andrzej Łukaszewski

Date: 03.09.2024

Github: `https://github.com/mikiSpoko200/gpu-bulwark`

---

# Agenda

<br/>

- Introduction (3min)
  * Type-Driven Design
  * Systems programming
  * Why Rust
  * Why OpenGL
  * Project's goals 
  * Key features

- Key features and examples (3min)
  * 

- Conclusions (1min)
  * What was achieved
  * Current limitations
  * Future plans

---

# Introduction

---

## Type-Driven Design

<br/>

- **Expressing Contracts with Types**: Using types to define precise contracts for data structures, functions and modules
- **Data Invariants**: Leveraging types to enforce constraints at compile-time, ensuring that certain properties hold throughout the program (e.g., positive numbers, non-empty lists)
- **Type-Safe APIs**: Representing data states explicitly with types along with specifying state transitions, making invalid application states unrepresentable
- **Static Property Enforcement**: Using types to express static properties (e.g., immutability, range restrictions) guaranteeing correctness at run-time

---

## Systems programming

<br/>

- **Robust solutions**: Creating programs resistant to misuse
- **System resource management**: Manual handling of limited system resources in a safe and efficient manner
- **Concurrency and Parallelism**: Working with need for explicit synchronization
- **Interoperability with External Components**: Facilitating communication with other software, hardware, and external systems - integration with different binary protocols
- **Performance Optimization**: Ensuring efficient, low-latency, and high-throughput execution

---

## Why Rust?

<br/>

Rust is a modern, statically typed, systems programming language.

At the same time it's a relatively new language (1.0 in 2015), the goal of this study was to showcase that it can be successfully applied to graphics programming.

Notable features:
* State of the art type system
* Natively compiled, with very good support for C interoperability
* Precise semantics of resource management
* **borrow checker** allows to cleanly express how different pieces of program interact with each other
* Large ecosystem of open-source packages

---

## Why OpenGL?

<br/>

OpenGL is a very mature, cross–platform, well understood and widely supported graphics specification.
It's arguably the best API for beginners in hardware-accelerated graphics programming.

However, it's also an old C API, with global state mutation at its core\* and very limited debugging facilities. 
Certain aspects of the programming interface make using OpenGL difficult despite its simple execution model. 

We felt like Rust's type system could mitigate some of these difficulties, providing an alternative for beginners to get into computer graphics
without spending weeks on getting used to writing OpenGL.

---

## Project's Goals and key features

<br/>

There were two main goals for this study:

* Explore type-driven design in Rust

* Implement a minimalistic OpenGL wrapper

---

## Explore type-driven design in Rust

<br/>

* Determine benefits and downsides of such an approach to systems programming

* Ascertain feasibility of developing software that way

* Identify common patterns for type-driven design

---

## Implement a minimalistic OpenGL wrapper

<br/>

* Remain as close to original specification as possible

* Implement only the most essential functionality

* Prevent as many ill-formed programs at compile-time as possible

* Finally, optimize UX aspects 

---

## Computer Graphics primer (in 30 seconds)

---

## API specifications for hardware accelerated graphics

<br/>

Graphics Processing Units (GPUs) are controlled via specification like OpenGL, Vulkan or DirectX.

GPU vendors implement specifications and software developers program against said specifications.

This makes code reusable across different devices.

---

## Programmable graphics pipeline

<br/>

Hardware rendering is quite a complicated process. It requires many sequential stages (hence pipeline); some of which are **programmable**.

Programs that specify the behavior of a programmable stage are called **shaders**.

Shaders are written in specialized languages like **GLSL** or **HLSL**, which don't directly control GPU's resources - they only declare what they expect to be given.
Thus, a need for **interoperability** arises since the programmer must manually guarantee that correct data is present at render-time.

---

## Shaders

<br/>

<div class="container">
<div class="col">

Vertex shader

```glsl[]
#version 420 core

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4(in_position, 1.0);
    color = vec4(in_color, 1.0);
}
```
</div>
<div class="col">

Fragment shader

```glsl[]
#version 420 core


layout(location = 0) in vec4 color;
layout(location = 0) out vec4 out_color;


void main() {
    out_color = color;
}

```
</div>
</div>

---

# Key features and examples

---

## GLSL DSL macros

<br/>

In order to facilitate seamless interoperability macros providing GLSL domain specific languages were developed.

<div class="container">
<div class="col">

```rust[]
type VertexShaderInputs = glsl::Glsl! {
    layout(location = 0) in vec3 in_position;
    layout(location = 1) in vec3 in_normal;
    layout(location = 2) in vec3 in_tex_coordinates;
};

fn update_uniforms(new_matrix: [[f32; 4]; 4]) {
    // ...
    let glsl::vars! [ mvp_matrix, dt ] = gb::glsl! {
        layout(location = 0) in mat4;
        layout(location = 1) in float;
    };
    program.uniform(&mvp_matrix, &new_matrix);
    // ...
}
```
</div>
<div class="col">

```glsl[]
#version 420 core

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_tex_coordinates;

// outputs ..

layout(location = 0) uniform mat4 mvp_matrix;
layout(location = 1) uniform float dt;

void main() {
    // ...
}
```
</div>
</div>

---

## Compile-time guarantees

<br/>

```glsl[]
#version 420 core

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4(in_position, 1.0);
    color = vec4(in_color, 1.0);
}
```

---

## 1. Data compatibility with shader declarations

<br/>

Keeping track of shape of a N-dimensional arrays is a tedious and error prone process.
Momentary lack of focus may cause invalid data to be provided to a routine.
This will usually cause a run-time error, but since OpenGL exposes a C API
and is provided as a binary for linking by the OS, it cannot use type information to 
bound check passed data.  

---

## 1. Data compatibility with shader declarations

<br/>

<div class="container">
<div class="col">

```c++[]
Buffer<int> colorBuffer = Buffer<float>::Array();
const std::vector<int> colors = {
    1, 0, 0, 1,
    0, 1, 0, 1,
    0, 0, 1, 1,
};

colorBuffer.Data(colors, GL_STATIC_DRAW);
vao.VertexAttribPointer(0, colorBuffer, 3, GL_FLOAT);
```
</div>
<div class="col">

```rust[]
let glsl::vars![ vin_color, vin_position ] = gb::glsl! {
    layout(location = 0) in vec3;
    layout(location = 1) in vec3;
};
let mut colors = Buffer::create();

colors.data::<(Dynamic, Draw)>(&[
    [1, 0, 0, 1], 
    [0, 1, 0, 1], 
    [0, 0, 1, 1i32]
]);

let vao: VertexArray<Attributes> = VertexArray::create()
    .vertex_attrib_pointer(&vin_color, colors);
```
</div>
</div>

---

## 1. Data compatibility with shader declarations

<br/>

```
error[E0308]: mismatched types
   --> src\listing.rs:101:44
    |
101 |           let vao: VertexArray<Attributes> = VertexArray::create()
    |  __________________-----------------------___^
    | |                  |
    | |                  expected due to this
102 | |             .vertex_attrib_pointer(&vin_color, colors)
103 | |             .vertex_attrib_pointer(&vin_position, positions)
    | |____________________________________________________________^ expected `f32`, found `i32`
    |
    = note: expected struct `VertexArray<(((), Attribute<[f32; 3], _>), Attribute<_, _>)>`
               found struct `VertexArray<(((), Attribute<[i32; 4], _>), Attribute<_, _>)>`
```

---

## 2. Valid resource locations

<br/>

Another bug strong typing can detect, is using attribute index
which does not match with vertex shader input variables.

```glsl[3-4]
#version 420 core

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4(in_position, 1.0);
    color = vec4(in_color, 1.0);
}
```

---

## 2. Valid resource locations

<br/>

<div class="container">
<div class="col">

```c++[2, 5]
colorBuffer.Data(colors, GL_STATIC_DRAW);
vao.VertexAttribPointer(0, colorBuffer, 3, GL_FLOAT);

positionBuffer.Data(positions, GL_STATIC_DRAW);
vao.VertexAttribPointer(0, positionBuffer, 3, GL_FLOAT);
```

</div>
<div class="col">

```rust[1-4,7-8]
let glsl::vars![ vin_color, vin_position ] = gb::glsl! {
    layout(location = 0) in vec3;
    layout(location = 1) in vec3;
};

let vao: VertexArray<Attributes> = VertexArray::create()
    .vertex_attrib_pointer(&vin_color, colors)
    .vertex_attrib_pointer(&vin_color, positions)
    ;
```
</div>
</div>

---

## 2. Valid resource locations

<br/>

```
error[E0308]: mismatched types
   --> src\listing.rs:105:36
    |
  8 |             .vertex_attrib_pointer(&vin_color, positions)
    |              --------------------- ^^^^^^^^^^ expected `1`, found `0`
    |              |
    |              arguments to this method are incorrect
    |
    = note: expected reference `&gpu_bulwark::glsl::Variable<_, gpu_bulwark::glsl::layout::Location<1>, _>`
               found reference `&gpu_bulwark::glsl::Variable<_, gpu_bulwark::glsl::layout::Location<0>, GVec<f32, 3>>`
```

---

## 3. Usage of only compiled shaders

<br/>

Type parameters along with Rust's method dispatch mechanism can be used to implement a type-level state machine.

```rust
mod ts {
    #[hi::mark(TypeState, Compilation)]
    pub enum Uncompiled { }

    #[hi::mark(TypeState, Compilation)]
    pub struct Compiled;
}
```

---

## 3. Usage of only compiled shaders

<br/>

```rust[]
impl Shader<ts::Uncompiled, ...> {
    /// Create a shader object, in uncompiled state.
    pub fn create() -> Self { ... }
    
    /// Add source code.
    pub fn source(&mut self, sources: &[&str]) -> &Self { ... }

    /// Try compiling.
    pub fn compile(self) -> Result<Shader<ts::Compiled, ...>, CompilationError> { ... }
}

impl Shader<ts::Compiled, ...> {
    pub fn into_main(self) -> Main<...> { ... }
}
```

---

## 3. Usage of only compiled shaders

<br/>

```rust[9]
impl Shader<ts::Uncompiled, ...> {
    /// Create a shader object, in uncompiled state.
    pub fn create() -> Self { ... }

    /// Add source code.
    pub fn source(&mut self, sources: &[&str]) -> &Self { ... }

    /// Try compiling.
    pub fn compile(self) -> Result<Shader<ts::Compiled, ...>, CompilationError> { ... }
}

impl Shader<ts::Compiled, ...> {
    pub fn into_main(self) -> Main<...> { ... }
}
```

---

## 3. Usage of only compiled shaders

<br/>

```rust[]
pub fn try_using_uncompiled() {
    let mut shader = shader::create::<shader::target::Vertex>();
    shader.into_main();
}
```

```
error[E0599]: no method named `into_main` found for struct `Shader<Uncompiled, Vertex, ()>` in the current scope
   --> src\example_wrong_buffers.rs:107:12
    |
  3 |     shader.into_main();
    |            ^^^^^^^^^ method not found in `Shader<Uncompiled, Vertex, ()>`
    |
    = note: the method was found for
            - `Shader<Compiled, T, Decls>`
```

---

## 4. Correct pipeline configuration

<br/>

<div class="container">
<div class="col">

```rust[]
let vs = uncompiled_vs
    .uniform(&view_matrix_location)
    .uniform(&scale_location)
    .compile()?
    .into_main()
    .inputs(&vs_inputs)
    .outputs(&vs_outputs);
let fs = uncompiled_fs
    .compile()?
    .into_main()
    .inputs(&fs_inputs)
    .output(&fs_output);
let common = common.compile()?.into_shared();
```

</div>
<div class="col">

```rust[]
let program = Program::builder()
    .uniforms(|definitions| definitions
        .define(&view_matrix_location, &matrix)
        .define(&scale_location, &scale)
    )
    .no_resources()
    .vertex_main(&vs)
    .uniforms(|matcher| matcher
        .bind(&scale_location)
        .bind(&view_matrix_location)
    )
    .vertex_shared(&common)
    .fragment_main(&fs)
    .build()?;
```
</div>
</div>

---

## Recursive types

<br/>

Rust associates types with their functionality using `impl` blocks.
These can be generic and apply to all types that satisfy given constraints.

```rust[]
pub MyTrait { }

impl<T> MyTrait for T where T: Debug { }
```

---

## Recursive types 

<br/>

```rust[]
trait HList {
    const SIZE: usize;
}

impl HList for () {
    const SIZE: usize = 0;
}

impl<Head, Tail> HList for (Head, Tail)
where
    Tail: HList
{
    const SIZE: usize = Tail::SIZE + 1;
}
```

---

## Recursive types - DSL macro expansion

<br/>

```rust[]
gb::glsl! {
    layout(location = 0) in mat4;
    layout(location = 4) in vec4;
};   
```

---

## Recursive types - DSL macro expansion

<br/>

```rust[]
gb::constraint::ValidExt::validated((
    (gb::constraint::ValidExt::validated((
        (),
        gb::glsl::variable::Variable::<
            gb::glsl::variable::storage::In,
            gb::glsl::variable::layout::Location<0>,
            gb::glsl::Mat4,
        >::default(),
    ))),
    gb::glsl::variable::Variable::<
        gb::glsl::variable::storage::In,
        gb::glsl::variable::layout::Location<4>,
        gb::glsl::Vec4,
    >::default(),
))
```

---

## Const parameterized types - DSL macro expansion

<br/>

```rust[6,12]
gb::constraint::ValidExt::validated((
    (gb::constraint::ValidExt::validated((
        (),
        gb::glsl::variable::Variable::<
            gb::glsl::variable::storage::In,
            gb::glsl::variable::layout::Location<0>,
            gb::glsl::Mat4,
        >::default(),
    ))),
    gb::glsl::variable::Variable::<
        gb::glsl::variable::storage::In,
        gb::glsl::variable::layout::Location<4>,
        gb::glsl::Vec4,
    >::default(),
))
```

---

# Conclusions

---

## OpenGL Wrapper 

<br/>

* Library implements the most essential parts of the OpenGL spec

* Library statically detects misuse of the OpenGL API

* Successful compilation guarantees correct configuration of the graphics pipeline

* Resulting code closely resembles analogous C / C++ code

* Type errors help to identify the cause of the problem

---

## Type-driven design

<br/>

* The biggest benefit of extensive use of the type system is the static verification of the correctness of the analyzed programs

* Type–driven design forces greater attention during the initial phase of software creation

* The logical principles underpinning the type system allow for a very intuitive description of
the program’s structure, algorithm invariants, as well as the interactions between different components

* Enforcing greater attention during system architecture design brings significant benefits in later stages of development

---

## Rust for OpenGL

<br/>

* Interfacing with OS platform will be inherently `unsafe`

* Rust is a viable technology for programming computer graphics

* With initial overhead of providing safe facades around raw OS APIs, development becomes simpler than in `C` or `C++`  

* Rust's type system can be used to express data formats to external APIs

---

## Innovative approach to computer graphics

<br/>

- Type-Safe API Design

- A new way to learn Graphics Programming

- Precision in Interoperability

- Novelty in Combining Disciplines

---

## Cross disciplinary
<br/>

* Programming `unsafe` Rust requires a great deal of attention, as Rust heavily exploits assumption of no Undefined Behavior

* Programming language theory - in-depth understating of Rust's type system and its limitations, studying numerous RFCs

* Software design - a great deal of attention was put towards designing API up to common engineering standards

---

## Current limitations
<br/>

* No `impl` specialization

* No negative trait bounds

* No const generics in const expressions

* No `const fn`s in traits

---

## Future plans

---

## Further improvements, community feedback

---

## Publishing to `crates.io`
<br/>

Source code for this project is already publicly available on github.
Next step in proliferating type driven design is publishing this crate to `crates.io`, once more features are added.

One module of this library called [`himark`](https://crates.io/crates/himark) was published during development,
and as of today has almost 1200 downloads.

---

## Integration with [`rust-gpu`](https://github.com/EmbarkStudios/rust-gpu)

```rust
use glam::{Vec3, Vec4, vec2, vec3};

#[spirv(fragment)]
pub fn main(
    #[spirv(frag_coord)] in_frag_coord: &Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
    let mut uv = (frag_coord - 0.5 * vec2(constants.width as f32, constants.height as f32))
        / constants.height as f32;
    uv.y = -uv.y;

    let eye_pos = vec3(0.0, 0.0997, 0.2);
let sun_pos = vec3(0.0, 75.0, -1000.0);
    let dir = get_ray_dir(uv, eye_pos, sun_pos);

    // evaluate Preetham sky model
    let color = sky(dir, sun_pos);

    *output = tonemap(color).extend(1.0)
}
```

---

## Thank you for your attention

<br/>

*Analysis of Type-Driven approach to systems programming: Implementation of OpenGL library for Rust*

<br/>

Author: Mikołaj Depta

Supervisor: dr. Andrzej Łukaszewski

Date: 03.09.2024

Github: `https://github.com/mikiSpoko200/gpu-bulwark`
