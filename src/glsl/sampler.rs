use std::marker::PhantomData;


pub struct Sampler<T>(PhantomData<T>) /* where T: Texture target  */;
