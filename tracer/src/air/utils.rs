use core::ops::{Mul, MulAssign, Sub};
use winterfell::math::FieldElement;

pub fn binary_flag<E>(expected: &str, test: &[E], one: E) -> E
where
    E: Mul<Output = E> + Sub<Output = E> + Copy + FieldElement,
{
    let mut result = one;
    for (i, bit) in expected.chars().enumerate() {
        result *= if bit == '1' { test[i] } else { one - test[i] };
    }
    result
}
