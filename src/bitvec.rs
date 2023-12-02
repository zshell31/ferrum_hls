use std::{
    cmp::Ordering,
    fmt::{self, Binary, Display, LowerHex},
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

use fhdl_macros::{blackbox, blackbox_ty};
use num_bigint::BigUint;
use num_traits::Zero;

use crate::{
    bitpack::{BitPack, IsPacked},
    cast::CastFrom,
    const_functions::{bit, slice},
    const_helpers::{Assert, IsTrue},
    signal::SignalValue,
};

// TODO: maybe union?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[blackbox_ty(BitVec)]
pub enum BitVec<const N: usize> {
    Short(u128),
    Long(BigUint),
}

impl<const N: usize> BitVec<N> {
    fn new_from_u128(val: u128) -> Self {
        match N.cmp(&128) {
            Ordering::Less => {
                let mask = (1 << N) - 1;
                Self::Short(val & mask)
            }
            Ordering::Equal => Self::Short(val),
            Ordering::Greater => {
                let mask = (BigUint::from(1_u8) << N) - 1_u8;
                Self::Long(BigUint::from(val) & mask)
            }
        }
    }

    fn new_from_biguint(val: BigUint) -> Self {
        match N.cmp(&128) {
            Ordering::Less => {
                let mask = (1 << N) - 1;
                Self::Short(u128::try_from(val).unwrap() & mask)
            }
            Ordering::Equal => Self::Short(u128::try_from(val).unwrap()),
            Ordering::Greater if val.bits() as usize == N => Self::Long(val),
            _ => {
                let mask = (BigUint::from(1_u8) << N) - 1_u8;
                Self::Long(val & mask)
            }
        }
    }

    fn bit_(&self, n: usize) -> bool {
        if n >= N {
            return false;
        }

        match self {
            Self::Short(short) => (short & (1 << n)) > 0,
            Self::Long(long) => long.bit(n as u64),
        }
    }

    pub(crate) fn is_non_zero(&self) -> bool {
        match self {
            Self::Short(short) => *short != 0,
            Self::Long(long) => !long.is_zero(),
        }
    }

    pub fn bit<const M: usize>(self) -> bool
    where
        Assert<{ bit(M, N) }>: IsTrue,
    {
        self.bit_(M)
    }

    pub fn msb(self) -> bool {
        self.bit_(N - 1)
    }

    #[blackbox(BitVecSlice)]
    pub fn slice<const S: usize, const M: usize>(self) -> BitVec<M>
    where
        Assert<{ slice(S, M, N) }>: IsTrue,
    {
        match self {
            Self::Short(short) => {
                let mask = (1 << M) - 1;
                BitVec::<M>::from((short >> S) & mask)
            }
            Self::Long(long) => {
                let mask = (BigUint::from(1_u8) << M) - 1_u8;
                BitVec::<M>::from((long >> S) & mask)
            }
        }
    }

    #[blackbox(BitVecUnpack)]
    pub fn unpack<T: BitPack<Packed = Self>>(self) -> T {
        T::unpack(self)
    }
}

impl<const N: usize> SignalValue for BitVec<N> {}

impl<const N: usize> IsPacked for BitVec<N> {
    #[inline(always)]
    fn zero() -> Self {
        Self::from(0_u8)
    }
}

impl<const N: usize> From<BigUint> for BitVec<N> {
    #[inline(always)]
    fn from(val: BigUint) -> Self {
        Self::new_from_biguint(val)
    }
}

impl<const N: usize, const M: usize> CastFrom<BitVec<M>> for BitVec<N> {
    fn cast_from(from: BitVec<M>) -> BitVec<N> {
        match from {
            BitVec::<M>::Short(short) => BitVec::<N>::from(short),
            BitVec::<M>::Long(long) => BitVec::<N>::from(long),
        }
    }
}

macro_rules! impl_from {
    ($( $prim:ty => $prim_bits:literal ),+) => {
        $(
            impl<const N: usize> From<$prim> for BitVec<N> {
                #[inline(always)]
                fn from(val: $prim) -> Self {
                    Self::new_from_u128(val as u128)
                }
            }

            impl<const N: usize> From<BitVec<N>> for $prim
            where
                Assert<{ N <= $prim_bits }>: IsTrue
            {
                #[inline(always)]
                fn from(val: BitVec<N>) -> Self {
                    match val {
                        BitVec::Short(short) => short as $prim,
                        BitVec::Long(_) => unreachable!()
                    }
                }
            }

        )+
    };
}

impl_from!(
    u8 => 8,
    u16 => 16,
    u32 => 32,
    u64 => 64,
    u128 => 128,
    usize => 64
);

impl<const N: usize> Display for BitVec<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Short(short) => Display::fmt(short, f),
            Self::Long(long) => Display::fmt(long, f),
        }
    }
}

impl<const N: usize> Binary for BitVec<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Short(short) => write!(f, "{:0width$b}", short, width = N),
            Self::Long(long) => write!(f, "{:0width$b}", long, width = N),
        }
    }
}

impl<const N: usize> LowerHex for BitVec<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = N.div_ceil(8);
        match self {
            Self::Short(short) => write!(f, "{:0width$x}", short, width = width),
            Self::Long(long) => write!(f, "{:0width$x}", long, width = width),
        }
    }
}

macro_rules! impl_op {
    ($trait:ident => $method:ident) => {
        impl<const N: usize> $trait for BitVec<N> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Short(lhs), Self::Short(rhs)) => lhs.$method(rhs).into(),
                    (Self::Long(lhs), Self::Long(rhs)) => lhs.$method(rhs).into(),
                    _ => unreachable!(),
                }
            }
        }
    };
    ($trait:ident => $method:ident => $spec_method:ident) => {
        impl<const N: usize> $trait for BitVec<N> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Short(lhs), Self::Short(rhs)) => lhs.$spec_method(rhs).into(),
                    (Self::Long(lhs), Self::Long(rhs)) => lhs.$method(rhs).into(),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! impl_ops_for_prim {
    ($trait:ident => $method:ident => $( $prim:ty ),+) => {
        $(
            impl<const N: usize> $trait<$prim> for BitVec<N> {
                type Output = BitVec<N>;

                fn $method(self, rhs: $prim) -> Self::Output {
                    self.$method(BitVec::<N>::from(rhs))
                }
            }

            impl<const N: usize> $trait<BitVec<N>> for $prim {
                type Output = BitVec<N>;

                fn $method(self, rhs: BitVec<N>) -> Self::Output {
                    BitVec::<N>::from(self).$method(rhs)
                }
            }

        )+
    };
}

macro_rules! impl_ops {
    ($( $trait:ident => $method:ident $( => $spec_method:ident )? ),+) => {
        $(
            impl_op!($trait => $method $( => $spec_method)?);

            impl_ops_for_prim!($trait => $method => u8, u16, u32, u64, u128, usize);
        )+
    };
}

impl_ops!(
    BitAnd => bitand,
    BitOr => bitor,
    BitXor => bitxor,
    Add => add => wrapping_add,
    Sub => sub => wrapping_sub,
    Mul => mul => wrapping_mul,
    Div => div => wrapping_div,
    Rem => rem => wrapping_rem
);

macro_rules! impl_shift_ops {
    ($( $prim:ty ),+) => {
        $(
            impl<const N: usize> Shl<$prim> for BitVec<N> {
                type Output = Self;

                fn shl(self, rhs: $prim) -> Self::Output {
                    match self {
                        Self::Short(short) => short.shl(rhs).into(),
                        Self::Long(long) => long.shl(rhs).into(),
                    }
                }
            }

            impl<const N: usize> Shr<$prim> for BitVec<N> {
                type Output = Self;

                fn shr(self, rhs: $prim) -> Self::Output {
                    match self {
                        Self::Short(short) => short.shr(rhs).into(),
                        Self::Long(long) => long.shr(rhs).into(),
                    }
                }
            }
        )+
    };
}

impl_shift_ops!(u8, u16, u32, u64, u128, usize);

impl<const N: usize> Not for BitVec<N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Short(short) => short.not().into(),
            Self::Long(long) => BigUint::from_slice(
                long.iter_u32_digits()
                    .map(|digit| !digit)
                    .collect::<Vec<u32>>()
                    .as_slice(),
            )
            .into(),
        }
    }
}
