use std::fmt::Debug;

mod sealed {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

pub trait Word: sealed::Sealed + Copy + Eq + Debug {
    const BITS: u32;
    const ZERO: Self;
    const ONES: Self;

    fn to_u64(self) -> u64;
    fn from_u64(value: u64) -> Self;
    fn count_ones(self) -> u32;
    fn bit_and(self, other: Self) -> Self;
    fn bit_or(self, other: Self) -> Self;
    fn bit_xor(self, other: Self) -> Self;
    fn bit_not(self) -> Self;
    fn is_uniform(self) -> bool {
        self == Self::ZERO || self == Self::ONES
    }
}

macro_rules! impl_word {
    ($($ty:ty),+$(,)?) => {
        $(
            impl Word for $ty {
                const BITS: u32 = <$ty>::BITS;
                const ZERO: Self = 0;
                const ONES: Self = <$ty>::MAX;

                fn to_u64(self) -> u64 {
                    u64::from(self)
                }

                fn from_u64(value: u64) -> Self {
                    <$ty>::try_from(value)
                        .expect("value does not fit in word")
                }

                fn count_ones(self) -> u32 {
                    <$ty>::count_ones(self)
                }

                fn bit_and(self, other: Self) -> Self {
                    self & other
                }

                fn bit_or(self, other: Self) -> Self {
                    self | other
                }
                fn bit_xor(self, other: Self) -> Self {
                    self ^ other
                }
                fn bit_not(self) -> Self {
                    !self
                }
            }
        )+
    };
}

impl_word!(u8, u16, u32, u64);
