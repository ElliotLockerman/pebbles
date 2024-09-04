
use core::ops::{Div, Rem};
use std::{fmt, ops};

use num_traits::ops::wrapping::*;
use num_traits::cast::FromPrimitive;
use num_traits::int::PrimInt;

macro_rules! wrapping_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        impl $trait_name for $t {
            #[inline]
            fn $method(&self, v: &Self) -> Self {
                <$t>::$method(*self, *v)
            }
        }
    };
}


pub trait WrappingDiv: Sized + Div<Self, Output = Self> {
    fn wrapping_div(&self, v: &Self) -> Self;
}

wrapping_impl!(WrappingDiv, wrapping_div, u8);
wrapping_impl!(WrappingDiv, wrapping_div, u16);
wrapping_impl!(WrappingDiv, wrapping_div, u32);
wrapping_impl!(WrappingDiv, wrapping_div, u64);
wrapping_impl!(WrappingDiv, wrapping_div, usize);
wrapping_impl!(WrappingDiv, wrapping_div, u128);
wrapping_impl!(WrappingDiv, wrapping_div, i8);
wrapping_impl!(WrappingDiv, wrapping_div, i16);
wrapping_impl!(WrappingDiv, wrapping_div, i32);
wrapping_impl!(WrappingDiv, wrapping_div, i64);
wrapping_impl!(WrappingDiv, wrapping_div, isize);
wrapping_impl!(WrappingDiv, wrapping_div, i128);


pub trait WrappingRem: Sized + Rem<Self, Output = Self> {
    fn wrapping_rem(&self, v: &Self) -> Self;
}

wrapping_impl!(WrappingRem, wrapping_rem, u8);
wrapping_impl!(WrappingRem, wrapping_rem, u16);
wrapping_impl!(WrappingRem, wrapping_rem, u32);
wrapping_impl!(WrappingRem, wrapping_rem, u64);
wrapping_impl!(WrappingRem, wrapping_rem, usize);
wrapping_impl!(WrappingRem, wrapping_rem, u128);
wrapping_impl!(WrappingRem, wrapping_rem, i8);
wrapping_impl!(WrappingRem, wrapping_rem, i16);
wrapping_impl!(WrappingRem, wrapping_rem, i32);
wrapping_impl!(WrappingRem, wrapping_rem, i64);
wrapping_impl!(WrappingRem, wrapping_rem, isize);
wrapping_impl!(WrappingRem, wrapping_rem, i128);


pub trait AsUnsigned {
    type Unsigned: Int;
    fn as_unsigned(&self) -> Self::Unsigned;
}

macro_rules! convert_impl {
    ($trait_name:ident, $typ:ty, $dest_type:ty) => {
        impl AsUnsigned for $typ {
            type Unsigned = $dest_type;
            fn as_unsigned(&self) -> Self::Unsigned {
                *self as Self::Unsigned
            }
        }
    }
}

convert_impl!(AsUnsigned, u8, u8);
convert_impl!(AsUnsigned, u16, u16);
convert_impl!(AsUnsigned, u32, u32);
convert_impl!(AsUnsigned, u64, u64);
convert_impl!(AsUnsigned, u128, u128);
convert_impl!(AsUnsigned, i8, u8);
convert_impl!(AsUnsigned, i16, u16);
convert_impl!(AsUnsigned, i32, u32);
convert_impl!(AsUnsigned, i64, u64);
convert_impl!(AsUnsigned, i128, u128);

pub trait Int = PrimInt + WrappingAdd + WrappingSub + WrappingMul + WrappingNeg + WrappingShl + WrappingShr + WrappingDiv + WrappingRem + FromPrimitive + fmt::Display + fmt::Octal + fmt::UpperHex + fmt::Binary + ops::ShrAssign + AsUnsigned;

