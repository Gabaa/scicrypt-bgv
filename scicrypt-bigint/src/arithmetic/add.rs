use std::{
    cmp::{max, min},
    iter::Sum,
    ops::{Add, AddAssign},
};

use gmp_mpfr_sys::gmp;

use crate::{scratch::Scratch, UnsignedInteger, GMP_NUMB_BITS};

// impl SignedInteger {
//     pub fn leaky_add_assign(&mut self, rhs: &Self) {
//         unsafe {
//             gmp::mpz_add(&mut self.value, &self.value, &rhs.value);
//         }

//         self.size_in_bits = self.significant_bits() as u32;
//     }
// }

impl AddAssign<&UnsignedInteger> for UnsignedInteger {
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert!(self.size_in_bits >= rhs.size_in_bits);

        let n = min(self.value.size, rhs.value.size);

        if n == 0 {
            return;
        }

        unsafe {
            let carry = gmp::mpn_add_n(
                self.value.d.as_mut(),
                self.value.d.as_ptr(),
                rhs.value.d.as_ptr(),
                n as i64,
            );

            let largest_size = max(self.value.size, rhs.value.size) as i32;

            self.value.size = largest_size + carry as i32;
            self.size_in_bits = max(self.size_in_bits, rhs.size_in_bits) + carry as u32;
        }
    }
}

impl Add<&UnsignedInteger> for UnsignedInteger {
    type Output = UnsignedInteger;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<u64> for UnsignedInteger {
    fn add_assign(&mut self, rhs: u64) {
        unsafe {
            let scratch_size =
                gmp::mpn_sec_add_1_itch(self.value.size as i64) as usize * GMP_NUMB_BITS as usize;

            let mut scratch = Scratch::new(scratch_size);

            let carry = gmp::mpn_sec_add_1(
                self.value.d.as_mut(),
                self.value.d.as_ptr(),
                self.value.size as i64,
                rhs,
                scratch.as_mut(),
            );

            self.value.size += carry as i32;
            self.size_in_bits += carry as u32;
        }
    }
}

impl Add<u64> for UnsignedInteger {
    type Output = UnsignedInteger;

    fn add(mut self, rhs: u64) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> Sum<&'a UnsignedInteger> for UnsignedInteger {
    fn sum<I: Iterator<Item = &'a UnsignedInteger>>(mut iter: I) -> Self {
        let initial = iter.next().unwrap().clone();
        iter.fold(initial, |x, y| x + y)
    }
}

#[cfg(test)]
mod tests {
    use crate::UnsignedInteger;

    #[test]
    fn test_addition() {
        let mut x = UnsignedInteger::from_string("5378239758327583290580573280735".to_string(), 10, 103);
        let y = UnsignedInteger::from_string("49127277414859531000011129".to_string(), 10, 86);

        x += &y;

        assert_eq!(
            UnsignedInteger::from_string("5378288885604998150111573291864".to_string(), 10, 103),
            x
        );
        assert_eq!(x.size_in_bits, 103);
    }

    #[test]
    fn test_addition_u64() {
        let mut x = UnsignedInteger::from_string("5378239758327583290580573280735".to_string(), 10, 103);
        let y = 14;

        x += y;

        assert_eq!(
            UnsignedInteger::from_string("5378239758327583290580573280749".to_string(), 10, 103),
            x
        );
        assert_eq!(x.size_in_bits, 103);
    }

    // #[test]
    // fn test_addition_negative() {
    //     let mut x = BigInteger::from_string("5378239758327583290580573280735".to_string(), 10, 103);
    //     let y = BigInteger::from_string("-49127277414859531000011129".to_string(), 10, 86);

    //     x += &y;

    //     assert_eq!(
    //         BigInteger::from_string("5378190631050168431049573269606".to_string(), 10, 103),
    //         x
    //     );
    //     assert_eq!(x.size_in_bits, 103);
    // }
}
