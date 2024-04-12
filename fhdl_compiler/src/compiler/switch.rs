use fhdl_netlist::{
    const_val::ConstVal,
    node::{Mux, MuxArgs},
};
use rustc_middle::{
    mir::{BasicBlock, Operand, SwitchTargets},
    ty::Ty,
};
use rustc_span::Span;
use tracing::{debug, error, instrument};

use super::{
    item::{Group, Item, ModuleExt},
    Compiler, Context, SymIdent,
};
use crate::error::{Error, SpanError, SpanErrorKind};

pub trait SwitchTargetsExt {
    type Value;

    fn variants(&self) -> impl Iterator<Item = (usize, BasicBlock)>;

    fn otherwise(&self) -> BasicBlock;

    fn value_for_target(&self, idx: usize) -> Self::Value;
}

impl SwitchTargetsExt for SwitchTargets {
    type Value = u128;

    fn variants(&self) -> impl Iterator<Item = (usize, BasicBlock)> {
        self.iter().map(|(_, target)| target).enumerate()
    }

    #[inline]
    fn otherwise(&self) -> BasicBlock {
        self.otherwise()
    }

    fn value_for_target(&self, idx: usize) -> Self::Value {
        self.iter().nth(idx).unwrap().0
    }
}

impl<'tcx> Compiler<'tcx> {
    #[instrument(level = "debug", skip(self, discr, targets, ctx, span))]
    pub fn visit_switch<Targets: SwitchTargetsExt<Value = u128>>(
        &mut self,
        switch_block: BasicBlock,
        discr: &Operand<'tcx>,
        targets: &Targets,
        ctx: &mut Context<'tcx>,
        span: Span,
    ) -> Result<Option<BasicBlock>, Error> {
        let mir = ctx.mir;
        let discr = self.visit_operand(discr, ctx, span)?;

        let convergent_block =
            self.find_convergent_block(switch_block, ctx)
                .ok_or_else(|| {
                    error!("cannot find convergent block for switch {switch_block:?}");
                    SpanError::new(SpanErrorKind::NotSynthSwitch, span)
                })?;

        if !ctx.locals.has_branches() {
            for (idx, target) in targets.variants() {
                if !mir.basic_blocks[target].is_empty_unreachable() {
                    ctx.locals.go_to_variant(idx);
                    self.visit_blocks(Some(target), Some(convergent_block), ctx)?;
                    ctx.locals.leave_branch();
                }
            }

            let otherwise = targets.otherwise();
            if !mir.basic_blocks[otherwise].is_empty_unreachable() {
                ctx.locals.go_to_otherwise();
                self.visit_blocks(Some(otherwise), Some(convergent_block), ctx)?;
                ctx.locals.leave_branch();
            }

            ctx.locals.collect_branch_locals(span)?;
            debug!("ctx.locals: {:#?}", ctx.locals);

            if !ctx.locals.branch_locals().is_empty() {
                let discr = ctx.module.get_discr(&discr);
                let discr_width = ctx.module[discr].width();

                let output_ty = Ty::new_tup_from_iter(
                    self.tcx,
                    ctx.locals
                        .branch_locals()
                        .iter()
                        .map(|local| mir.local_decls[*local].ty),
                );
                let output_ty = self.resolve_ty(output_ty, ctx.generic_args, span)?;

                let default = ctx.locals.otherwise().map(|otherwise| {
                    let item = Item::new(output_ty, Group::new(otherwise));
                    assert_eq!(output_ty.nodes(), item.nodes());
                    item.iter()
                });

                let variants = ctx.locals.variants().map(|(target_idx, locals)| {
                    let discr_val = targets.value_for_target(target_idx);
                    let item = Item::new(output_ty, Group::new(locals));
                    assert_eq!(output_ty.nodes(), item.nodes());
                    (ConstVal::new(discr_val, discr_width), item.iter())
                });

                let mux = ctx.module.add::<_, Mux>(MuxArgs {
                    outputs: output_ty.iter().map(|ty| (ty, None)),
                    sel: discr,
                    variants,
                    default,
                });
                let node_span = self
                    .span_to_string(span, ctx.fn_did)
                    .map(|span| format!("{span} ({switch_block:?})"));
                ctx.module.add_span(mux, node_span);

                let mux = ctx.module.combine_from_node(mux, output_ty);

                ctx.module
                    .assign_names_to_item(SymIdent::Mux.as_str(), &mux, false);

                ctx.locals.assign_branch_locals(mux);
            }

            if ctx.locals.is_root() {
                ctx.locals.prune_all_branches();
            }
        }

        Ok(Some(convergent_block))
    }
}
