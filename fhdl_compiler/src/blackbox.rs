pub mod array;
pub mod bin_op;
pub mod bitpack;
pub mod bitvec;
pub mod cast;
pub mod reg;
pub mod signal;
pub mod un_op;

use fhdl_common::BlackboxKind;
use fhdl_netlist::node::BinOp;
use rustc_hir::def_id::DefId;
use rustc_span::Span;

use crate::{
    compiler::{item::Item, item_ty::ItemTy, Compiler, Context},
    error::Error,
};

pub trait EvalExpr<'tcx> {
    fn eval(
        &self,
        compiler: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        output_ty: ItemTy<'tcx>,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Item<'tcx>, Error>;
}

#[derive(Debug)]
pub struct Blackbox {
    pub kind: BlackboxKind,
    pub fn_did: DefId,
}

macro_rules! eval_expr {
    (
        $( $blackbox_kind:ident => $eval:expr ),+ $(,)?
    ) => {
        impl<'tcx> Blackbox {
            pub fn eval(
                &self,
                compiler: &mut Compiler<'tcx>,
                args: &[Item<'tcx>],
                output_ty: ItemTy<'tcx>,
                ctx: &mut Context<'tcx>,
                span: Span,
            ) -> Result<Item<'tcx>, Error> {
                match self.kind {
                    $(
                        BlackboxKind::$blackbox_kind => $eval.eval(compiler,  args, output_ty, ctx, span),
                    )+
                }
            }
        }
    };
}

macro_rules! args {
    ($args:ident as $( $arg:pat ),+) => {
        let [$($arg,)+ ..] = $args else { panic!("not enough arguments"); };
    };
}

use args;

struct PassReceiver;

impl<'tcx> EvalExpr<'tcx> for PassReceiver {
    fn eval(
        &self,
        _: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        _: ItemTy<'tcx>,
        _: &mut Context<'tcx>,
        _: Span,
    ) -> Result<Item<'tcx>, Error> {
        args!(args as rec);

        Ok(rec.clone())
    }
}

eval_expr!(
    ArrayChain => array::Chain,

    BitPackPack => bitpack::Pack,
    BitPackUnpack => bitpack::Unpack,

    Bundle => PassReceiver,
    Unbundle => PassReceiver,

    OpEq => bin_op::BinOp(BinOp::Eq),
    OpNe => bin_op::BinOp(BinOp::Ne),
    OpLt => bin_op::BinOp(BinOp::Lt),
    OpLe => bin_op::BinOp(BinOp::Le),
    OpGt => bin_op::BinOp(BinOp::Gt),
    OpGe => bin_op::BinOp(BinOp::Ge),
    OpBitAnd => bin_op::BinOp(BinOp::BitAnd),
    OpBitOr => bin_op::BinOp(BinOp::BitOr),
    OpBitXor => bin_op::BinOp(BinOp::BitXor),
    OpAnd => bin_op::BinOp(BinOp::And),
    OpOr => bin_op::BinOp(BinOp::Or),
    OpAdd => bin_op::BinOp(BinOp::Add),
    OpSub => bin_op::BinOp(BinOp::Sub),
    OpMul => bin_op::BinOp(BinOp::Mul),
    OpDiv => bin_op::BinOp(BinOp::Div),
    OpRem => bin_op::BinOp(BinOp::Rem),
    OpShl => bin_op::BinOp(BinOp::Sll),
    OpShr => bin_op::BinOp(BinOp::Slr),
    OpNot => un_op::BitNot,

    CastFrom => cast::CastFrom,

    Index => bitvec::Slice { only_one: true },
    Slice => bitvec::Slice { only_one: false },

    RegEn => reg::RegEn { comb: false },
    RegEnComb => reg::RegEn { comb: true },

    SignalAndThen => signal::AndThen,
    SignalApply2 => signal::Apply2,
    SignalMap => signal::Map,
    SignalDff => signal::SignalDff { comb: false },
    SignalDffComb => signal::SignalDff { comb: true },
    SignalValue => PassReceiver,
    IntoSignal => PassReceiver,

    StdClone => PassReceiver,
);
