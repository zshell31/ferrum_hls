use fhdl_netlist::node::{DFFArgs, TyOrData, DFF};
use rustc_span::Span;

use super::{args, EvalExpr};
use crate::{
    compiler::{
        item::{Item, ModuleExt},
        item_ty::ItemTy,
        Compiler, Context, SymIdent,
    },
    error::Error,
};

pub struct SignalReg;

impl<'tcx> EvalExpr<'tcx> for SignalReg {
    fn eval(
        &self,
        compiler: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        output_ty: ItemTy<'tcx>,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Item<'tcx>, Error> {
        args!(args as clk, rst, en, init, comb);

        let clk = clk.port();
        let rst = ctx.module.to_bitvec(rst).port();
        let en = ctx.module.to_bitvec(en).port();
        let init = ctx.module.to_bitvec(init).port();

        let dff = ctx.module.add_and_get_port::<_, DFF>(DFFArgs {
            clk,
            rst: Some(rst),
            en: Some(en),
            init,
            data: TyOrData::Ty(output_ty.to_bitvec()),
            sym: SymIdent::Reg.into(),
        });
        let dff_out = ctx.module.from_bitvec(dff, output_ty);

        let comb = compiler.instantiate_closure(comb, &[dff_out.clone()], ctx, span)?;
        assert_eq!(comb.ty, output_ty);

        let comb_out = ctx.module.to_bitvec(&comb).port();
        DFF::set_data(&mut ctx.module, dff.node, comb_out);

        Ok(dff_out)
    }
}

pub struct SignalMap;

impl<'tcx> EvalExpr<'tcx> for SignalMap {
    fn eval(
        &self,
        compiler: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        _: ItemTy<'tcx>,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Item<'tcx>, Error> {
        args!(args as rec, comb);

        compiler.instantiate_closure(comb, &[rec.clone()], ctx, span)
    }
}

pub struct SignalAndThen;

impl<'tcx> EvalExpr<'tcx> for SignalAndThen {
    fn eval(
        &self,
        compiler: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        _: ItemTy<'tcx>,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Item<'tcx>, Error> {
        args!(args as rec, comb);

        compiler.instantiate_closure(comb, &[rec.clone()], ctx, span)
    }
}

pub struct SignalApply2;

impl<'tcx> EvalExpr<'tcx> for SignalApply2 {
    fn eval(
        &self,
        compiler: &mut Compiler<'tcx>,
        args: &[Item<'tcx>],
        _: ItemTy<'tcx>,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Item<'tcx>, Error> {
        args!(args as arg1, arg2, comb);

        compiler.instantiate_closure(comb, &[arg1.clone(), arg2.clone()], ctx, span)
    }
}
