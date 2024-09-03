
use core::ops::{Div, Rem};

macro_rules! wrapping_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        impl $trait_name for $t {
            #[inline]
            fn $method(&self, v: &Self) -> Self {
                <$t>::$method(*self, *v)
            }
        }
    };
    ($trait_name:ident, $method:ident, $t:ty, $rhs:ty) => {
        impl $trait_name<$rhs> for $t {
            #[inline]
            fn $method(&self, v: &$rhs) -> Self {
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


pub trait WrappingRem: Sized + Rem<Self, Output = Self> {
    fn wrapping_rem(&self, v: &Self) -> Self;
}

wrapping_impl!(WrappingRem, wrapping_rem, u8);
wrapping_impl!(WrappingRem, wrapping_rem, u16);
wrapping_impl!(WrappingRem, wrapping_rem, u32);
wrapping_impl!(WrappingRem, wrapping_rem, u64);
wrapping_impl!(WrappingRem, wrapping_rem, usize);
wrapping_impl!(WrappingRem, wrapping_rem, u128);
