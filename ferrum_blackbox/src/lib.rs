use strum::{Display, EnumString};

#[derive(Display, Debug, Clone, Copy, EnumString, PartialEq, Eq, Hash)]
pub enum Blackbox {
    ArrayMap,
    ArrayReverse,

    BitH,
    BitL,

    BitPackMsb,
    BitPackPack,
    BitPackRepack,

    BitVecShrink,
    BitVecSlice,
    BitVecUnpack,

    Bundle,
    Unbundle,

    Cast,

    SignalAnd,
    SignalAndThen,
    SignalApply2,
    SignalEq,
    SignalLift,
    SignalMap,
    SignalOr,
    SignalReg,
    SignalRegEn,
    SignalReset,
    SignalValue,
    SignalWatch,

    UnsignedIndex,

    StdClone,
    StdFrom,
    StdInto,
}

impl Blackbox {
    pub fn is_std_conversion(&self) -> Option<bool> {
        match self {
            Self::StdFrom => Some(true),
            Self::StdInto => Some(false),
            _ => None,
        }
    }
}

#[derive(Display, Debug, Clone, Copy, EnumString, PartialEq, Eq, Hash)]
pub enum BlackboxTy {
    Signal,
    Wrapped,
    BitVec,
    Bit,
    Clock,
    Unsigned,
    UnsignedMatch,
    Array,
}

impl BlackboxTy {
    pub fn is_match(&self) -> bool {
        matches!(self, BlackboxTy::UnsignedMatch)
    }
}
