
const N: usize = 1024;

pub struct Location<const N: usize>;

trait AllDiff {
    const ARRAY: [i64; N];
    const INDEX: usize;
}

impl<const FIRST: usize> AllDiff for ((), Location<FIRST>) {
    const INDEX: usize = 0;
    const ARRAY: [i64; N] = all_diff([-1; N], 0, FIRST);
}

impl<H, const PREV_LOCATION: usize, const CURR_LOCATION: usize> AllDiff for ((H, Location<PREV_LOCATION>), Location<CURR_LOCATION>)
where 
    (H, Location<PREV_LOCATION>): AllDiff
{
    const INDEX: usize = <(H, Location<PREV_LOCATION>)>::INDEX + 1;
    const ARRAY: [i64; N] = all_diff(<(H, Location<PREV_LOCATION>)>::ARRAY, Self::INDEX, CURR_LOCATION);
}

const fn all_diff(array: [i64; N], index: usize, location: usize) -> [i64; N] {
    if index > N {
        panic!("const buffer too small, please increase N");
    }

    let mut i = 0;
    let mut new_array = [-1; N];
    while i < index {
        if array[i] == location as _ {
            panic!("locations are overlapping");
        }
        new_array[i] = array[i];
        i += 1;
    }
    new_array[index] = location as _;
    new_array
}

macro_rules! locations {
    ($($locations:literal),+ $(,)?) => {
        locations!(@ () => $($locations),+)
    };
    (@ $acc:expr => $location:literal) => {
        ($acc, Location::<$location>)
    };
    (@ $acc:expr => $location:literal, $($tail:literal),*) => {
        locations!(@ ($acc, Location::<$location>) => $($tail),*)
    }
}

fn assert_all_different<T: AllDiff>(_: T) {
    let _ = T::ARRAY;
}

fn main() {
    let locations = locations!(1, 2, 3);
    let _ = assert_all_different(locations);
    // println!("{:?}", &<(((), Location<1>), Location<1>)>::ARRAY[..2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_1() { assert_all_different(locations!(1, 2, 3)); }

    #[test]
    fn test_valid_2() { assert_all_different(locations!(3, 2, 1)); }    
    
    #[test]
    fn test_valid_3() { assert_all_different(locations!(1, 3, 2)); }

    #[test]
    fn test_invalid_1() { assert_all_different(locations!(1, 1)); }
}