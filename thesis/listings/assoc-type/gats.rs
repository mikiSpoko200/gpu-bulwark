pub trait Foo {
    type Bar<T>: Clone;
    type Baz<T> where T: Default;
}