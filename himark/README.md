# Oh, hi `mark`.

For those who crave more ergonomic marker traits.

## Introduction
Marker traits are a common design pattern in Rust, used to denote certain properties or capabilities of types without requiring any additional implementation. However, managing marker traits can be tedious and often leads to boilerplate code. The `himark` crate aims to alleviate these issues by providing ergonomic utilities for working with marker traits.

## About
The `himark` crate simplifies the usage of marker traits in Rust by offering two main features:

1. **Automatic Implementation Generation**: Use `himark::mark` to automatically generate `impl` blocks for marker traits.
2. **Trait Validation**: Use `himark::marker` to ensure that a trait meets the criteria for being a marker trait.

## Usage

### Generating Implementations for Marker Traits
The `himark::mark` attribute macro generates implementations for specified marker traits, reducing the need for boilerplate code.

#### Example:
```rust
use himark::mark;

#[mark(MyMarkerTrait)]
struct MyStruct;
```

This will automatically generate the following implementation:
```rust
impl MyMarkerTrait for MyStruct {}
```

### Validating Marker Traits
The `himark::marker` attribute macro validates that a trait meets the criteria of being a marker trait, ensuring that it has no associated items and that all its super traits are also markers or auto traits.

#### Example:
```rust
use himark::marker;

#[marker]
trait MyMarkerTrait {}
```

This macro will produce a compile-time error if the trait does not meet the criteria for being a marker trait.

### Recommended configuration

For best user experience we recommend importing `himark` as `hi` either with `use himark as hi;` or custom `Cargo.toml` configuration.

```toml
[dependencies]
hi = { package = "himark", version = ... }
```

And write code as allows:

```rust
use himark as hi;

#[hi::mark(...)]
struct Foo { }
```

## Features
- **Automatic Implementation Generation**: Simplifies the process of implementing marker traits.
- **Trait Validation**: Ensures that your marker traits conform to the expected structure.

## Contributing
Contributions are welcome! Please feel free to submit a pull request or open an issue on GitHub.

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
