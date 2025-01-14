use std::io;

use fhdl_macros::impl_tuple_traits;

use crate::{
    bitpack::{BitPack, BitSize, BitVec, IsPacked},
    bundle::{Bundle, Unbundle},
    cast::{Cast, CastFrom},
    domain::ClockDomain,
    eval::{Eval, EvalCtx},
    signal::{Signal, SignalValue},
    trace::{IdCode, TraceVars, Traceable, Tracer},
};

impl_tuple_traits!(1);
impl_tuple_traits!(2);
impl_tuple_traits!(3);
impl_tuple_traits!(4);
impl_tuple_traits!(5);
impl_tuple_traits!(6);
impl_tuple_traits!(7);
impl_tuple_traits!(8);
impl_tuple_traits!(9);
impl_tuple_traits!(10);
impl_tuple_traits!(11);
impl_tuple_traits!(12);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        array::Array,
        bit::Bit,
        domain::{Clock, TD4},
        signal::SignalIterExt,
        unsigned::U,
    };

    #[test]
    fn unbundle() {
        let clk = Clock::<TD4>::new();
        let s: Signal<TD4, (U<4>, Bit)> = [
            (0_u8, false),
            (1, true),
            (2, true),
            (3, false),
            (4, true),
            (5, false),
        ]
        .into_iter()
        .map(Cast::cast)
        .into_signal();

        let res = s.unbundle();

        assert_eq!(
            res.eval(&clk)
                .take(6)
                .map(Cast::cast::<(u8, bool)>)
                .collect::<Vec<_>>(),
            [
                (0, false),
                (1, true),
                (2, true),
                (3, false),
                (4, true),
                (5, false),
            ]
        );
    }

    #[test]
    fn bundle() {
        let clk = Clock::<TD4>::new();
        let s: (Signal<TD4, U<4>>, Signal<TD4, Bit>) = (
            [0_u8, 1, 2, 3, 4, 5]
                .into_iter()
                .map(Cast::cast)
                .into_signal(),
            [false, true, true, false, true, false]
                .into_iter()
                .map(Cast::cast)
                .into_signal(),
        );

        let res = s.bundle();

        assert_eq!(
            res.eval(&clk)
                .take(6)
                .map(Cast::cast::<(u8, bool)>)
                .collect::<Vec<_>>(),
            [
                (0, false),
                (1, true),
                (2, true),
                (3, false),
                (4, true),
                (5, false),
            ]
        );
    }

    #[test]
    fn pack() {
        let s: (U<4>, Bit, Array<2, U<2>>) =
            (12_u8.cast(), false.cast(), [1_u8.cast(), 3_u8.cast()]);

        assert_eq!(s.pack(), 0b110000111_u64.cast::<BitVec<_>>());
    }

    #[test]
    fn unpack() {
        let s: (U<4>, Bit, Array<2, U<2>>) = BitPack::unpack(0b110000111_u64.cast());

        assert_eq!(s, (12_u8.cast(), false.cast(), [1_u8.cast(), 3_u8.cast()]));
    }
}
