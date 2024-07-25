use std::marker::PhantomData;

trait Disjoint {
    type Discriminant;
}

trait Type: Location { }

pub struct Vec<T, const N: usize>(PhantomData<T>) where T: Type;

impl<T, const N: usize> Disjoint for Vec<T, N> where T: Type {
    type Discriminant = T;
}

impl Location for i32 {
    const SIZE: usize = 1;
}

impl Location for f32 {
    const SIZE: usize = 1;
}

impl Location for f64 {
    const SIZE: usize = 1;
}

impl Type for i32 { }
impl Type for f32 { }
impl Type for f64 { }


trait Location {
    const SIZE: usize = 1;
}

mod private {
    use super::*;
    
    impl<T, const N: usize> Location for Vec<T, N> where Self: LocationHelper<<Self as Disjoint>::Discriminant>, T: Type {
        const SIZE: usize = <Self as LocationHelper<<Self as Disjoint>::Discriminant>>::SIZE;
    }
    
    trait LocationHelper<T> {
        const SIZE: usize;
    }
    
    impl<T, const N: usize> LocationHelper<i32> for Vec<T, N> where Self: Disjoint<Discriminant=i32>, T: Type {
        const SIZE: usize = 1;
    }
    
    impl<T, const N: usize> LocationHelper<f32> for Vec<T, N> where Self: Disjoint<Discriminant=f32>, T: Type {
        const SIZE: usize = 1;
    }
    
    impl<T, const N: usize> LocationHelper<f64> for Vec<T, N> where Self: Disjoint<Discriminant=f64>, T: Type {
        const SIZE: usize = match N {
            2 => 1,
            3 | 4 => 2,
            _ => panic!("unreachable"),
        };
    }
}

// impl<T, const N: usize> Type for Vec<T, N> where T: Type { }

fn main() {
    println!("f32: {}, i32: {}, f64: {}", Vec::<f32, 4>::SIZE, Vec::<i32, 4>::SIZE, Vec::<f64, 4>::SIZE);
}


