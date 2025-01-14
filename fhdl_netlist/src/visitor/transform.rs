use std::iter;

use fhdl_data_structures::{
    cursor::Cursor,
    graph::{NodeId, Port},
    FxHashMap,
};
use smallvec::SmallVec;

use crate::{
    cfg::InlineMod,
    const_val::ConstVal,
    netlist::{Module, ModuleId, NetList},
    node::{
        BinOpInputs, Const, ConstArgs, DFFArgs, DFFInputs, IsNode, MultiConst, NodeKind,
        SwitchInputs, TyOrData, DFF,
    },
    with_id::WithId,
};

const NODES_LIMIT_TO_INLINE: usize = 10;

pub struct Transform<'n> {
    netlist: &'n NetList,
    cons: FxHashMap<(ModuleId, ConstVal), Port>,
    max_inlines: Option<MaxInlines>,
}

pub struct MaxInlines {
    max: usize,
    current: usize,
}

impl MaxInlines {
    fn new(max_inlines: usize) -> Self {
        Self {
            max: max_inlines,
            current: 0,
        }
    }

    fn should_inline(&self) -> bool {
        self.current < self.max
    }

    fn inc(&mut self) {
        self.current += 1;
    }
}

impl<'n> Transform<'n> {
    pub fn new(netlist: &'n NetList) -> Self {
        Self {
            netlist,
            cons: Default::default(),
            max_inlines: netlist.cfg().max_inlines.map(MaxInlines::new),
        }
    }

    pub fn run(mut self) {
        if let Some(top) = self.netlist.top {
            self.visit_module(top);
        }
    }

    fn should_inline(&self) -> bool {
        self.max_inlines
            .as_ref()
            .map(|max_inlines| max_inlines.should_inline())
            .unwrap_or(true)
    }

    fn inc_inlines(&mut self) {
        if let Some(max_inlines) = &mut self.max_inlines {
            max_inlines.inc();
        }
    }

    fn visit_module(&mut self, mod_id: ModuleId) {
        let module = self.netlist.module(mod_id);
        let mut module = module.borrow_mut();

        let mut nodes = module.nodes();

        while let Some(node_id) = nodes.next_(&module) {
            if let Some(mod_inst) = module[node_id].mod_inst() {
                let mod_id = mod_inst.mod_id;

                // transform nodes of module before inlining module
                self.visit_module(mod_id);
            }

            let should_inline =
                self.transform(module.as_deref_mut(), node_id) && self.should_inline();

            if should_inline {
                let node_id = self.netlist.inline_mod(module.as_deref_mut(), node_id);
                self.inc_inlines();

                if let Some(node_id) = node_id {
                    nodes.set_next(node_id);
                }
            }
        }
    }

    fn transform(
        &mut self,
        mut module: WithId<ModuleId, &mut Module>,
        node_id: NodeId,
    ) -> bool {
        let node = module.node(node_id);

        let mut inline = false;
        match node.kind() {
            NodeKind::Pass(pass) => {
                let pass = node.with(pass);
                match module.to_const(pass.input(&module)) {
                    Some(const_val) => {
                        let output = pass.output[0];

                        module.replace::<_, Const>(node_id, ConstArgs {
                            ty: output.ty,
                            value: const_val.val(),
                            sym: output.sym,
                        });
                    }
                    None => {
                        if !(module.is_mod_output(Port::new(node_id, 0))
                            || module.is_mod_input(pass.input(&module)))
                        {
                            let pass = node.with(pass);
                            let input_ty = module[pass.input(&module)].ty;
                            let output_ty = pass.output[0].ty;
                            if input_ty.width() == output_ty.width() {
                                module.reconnect(node_id);
                            }
                        }
                    }
                }
            }
            NodeKind::Const(cons) => {
                self.eliminate_const(cons.value(), Port::new(node_id, 0), module);
            }
            NodeKind::MultiConst(_) => {
                self.eliminate_multi_const(node_id, module);
            }
            NodeKind::ModInst(mod_inst) => {
                let orig_module = self.netlist[mod_inst.mod_id].borrow();

                if orig_module.has_const_outputs() {
                    let const_args = orig_module.mod_outputs().iter().map(|port| {
                        let const_val = orig_module.to_const(*port).unwrap();
                        let port = orig_module[*port];

                        ConstArgs {
                            ty: port.ty,
                            value: const_val.val(),
                            sym: port.sym,
                        }
                    });

                    self.replace_with_multi_const(node_id, module, const_args);
                } else {
                    match self.netlist.cfg().inline_mod {
                        InlineMod::All => {
                            inline = true;
                        }
                        InlineMod::Auto => {
                            inline = orig_module.inline
                                || module.mod_in_count() == 0
                                || module.mod_out_count() == 0
                                || module.node_count() <= NODES_LIMIT_TO_INLINE
                                || module.node_has_const_inputs(node_id)
                        }
                        InlineMod::None => {
                            inline = false;
                        }
                    };
                }
            }
            NodeKind::BitNot(bit_not) => {
                if let Some(const_val) =
                    module.to_const(node.with(bit_not).input(&module))
                {
                    let const_val = !const_val;
                    let output = bit_not.output[0];
                    self.replace_with_const(node_id, module, ConstArgs {
                        ty: output.ty,
                        value: const_val.val(),
                        sym: output.sym,
                    });
                }
            }

            NodeKind::BinOp(bin_op) => {
                let BinOpInputs { lhs, rhs } = node.with(bin_op).inputs(&module);

                if let (Some(left), Some(right)) =
                    (module.to_const(lhs), module.to_const(rhs))
                {
                    let const_val = left.eval_bin_op(right, bin_op.bin_op);
                    let output = bin_op.output[0];

                    self.replace_with_const(node_id, module, ConstArgs {
                        ty: output.ty,
                        value: const_val.val(),
                        sym: output.sym,
                    });
                }
            }
            NodeKind::Splitter(splitter) => {
                let splitter = node.with(splitter);

                if splitter.pass_all_bits(&module) {
                    module.reconnect(node_id);
                } else {
                    let indices = splitter.eval_indices(&module);
                    let input = splitter.input(&module);
                    let input_val = module.to_const(input);

                    if let Some(input_val) = input_val {
                        let input_val = input_val.val();
                        let const_args = indices
                            .map(|(index, output)| {
                                let value =
                                    ConstVal::new(input_val >> index, output.width())
                                        .val();

                                ConstArgs {
                                    ty: output.ty,
                                    value,
                                    sym: output.sym,
                                }
                            })
                            .collect::<SmallVec<[ConstArgs; 1]>>()
                            .into_iter();

                        self.replace_with_multi_const(node_id, module, const_args);
                    } else {
                        drop(indices);

                        let input_id = splitter.input(&module).node;
                        let input = &module[input_id];

                        if let NodeKind::Merger(merger) = input.kind() {
                            if splitter.rev != merger.rev
                                && module.is_reversible(input_id, node_id)
                            {
                                module
                                    .reconnect_from_inputs_to_outputs(input_id, node_id);
                            }
                        }
                    }
                }
            }
            NodeKind::Merger(merger) => {
                // let sym = output.sym;

                // if !*rev {
                //     // for case:
                //     // ```verilog
                //     // wire [3:0] in;
                //     // wire [3:0] out;
                //     // assing out = {in[3], in[2], in[1], in[0]};
                //     // ```
                //     //
                //     // after transform:
                //     // ```verilog
                //     // wire [3:0] in;
                //     // wire [3:0] out;
                //     // assign out = in;
                //     // ```
                //     if let Some(node_out_id) = self.is_merger_eq_input(inputs) {
                //         let out = &self[node_out_id];
                //         return Pass::new(out.ty, node_out_id, sym).into();
                //     }
                // }

                let mut val = Some(ConstVal::new(0, 0));
                node.with(merger).inputs(&module).for_each(|input| {
                    match module.to_const(input) {
                        Some(new_val) => {
                            if let Some(val) = val.as_mut() {
                                val.shift(new_val);
                            }
                        }
                        None => {
                            val = None;
                        }
                    }
                });

                if let Some(const_val) = val {
                    let output = merger.output[0];
                    self.replace_with_const(node_id, module, ConstArgs {
                        ty: output.ty,
                        value: const_val.val(),
                        sym: output.sym,
                    });
                }
            }

            NodeKind::Extend(extend) => {
                let extend = node.with(extend);
                let output = extend.output[0];
                let input = extend.input(&module);

                match module.to_const(input) {
                    Some(const_val) => {
                        self.replace_with_const(node_id, module, ConstArgs {
                            ty: output.ty,
                            value: const_val.val(),
                            sym: output.sym,
                        });
                    }
                    None => {
                        if module[input].width() == output.width() {
                            module.reconnect(node_id);
                        }
                    }
                }
            }

            NodeKind::Switch(mux) => {
                let cases_len = mux.cases.len();
                let mux = node.with(mux);

                let chunk = {
                    let SwitchInputs { sel, cases, .. } = mux.inputs(&module);

                    let mut cases_ref = cases.into_iter();
                    let chunk = if cases_len == 1 {
                        Some(cases_ref.next().unwrap().1)
                    } else {
                        module.to_const(sel).and_then(|sel| {
                            for (case, chunk) in cases_ref {
                                if case.is_match(sel) {
                                    return Some(chunk);
                                }
                            }

                            None
                        })
                    };

                    chunk.map(|chunk| chunk.collect::<SmallVec<[_; 1]>>())
                };

                if let Some(chunk) = chunk {
                    module.reconnect_all_outgoing(node_id, chunk);
                }
            }

            NodeKind::DFF(dff) => {
                let dff = node.with(dff);
                let DFFInputs {
                    clk,
                    mut rst,
                    mut en,
                    init,
                    data,
                } = dff.inputs(&module);

                let mut replace = false;

                let mut true_rst = false;
                if let Some(const_val) = rst.and_then(|rst| module.to_const(rst)) {
                    if dff.rst_pol.bool(const_val.val() == 0) {
                        rst = None;
                        replace = true;
                    } else {
                        true_rst = true;
                    }
                }

                let mut false_en = false;
                if let Some(const_val) = en.and_then(|en| module.to_const(en)) {
                    if const_val.val() > 0 {
                        en = None;
                        replace = true;
                    } else {
                        false_en = true;
                    }
                };

                if replace {
                    let rst_kind = dff.rst_kind;
                    let rst_pol = dff.rst_pol;
                    let sym = dff.output[0].sym;

                    module.replace::<_, DFF>(node_id, DFFArgs {
                        rst_kind,
                        rst_pol,
                        clk,
                        rst,
                        en,
                        init,
                        data: TyOrData::Data(data),
                        sym,
                    });
                } else if true_rst || false_en {
                    module.reconnect_all_outgoing(node_id, iter::once(init));
                }
            }
            _ => {}
        };

        inline
    }

    fn replace_with_const(
        &mut self,
        node_id: NodeId,
        mut module: WithId<ModuleId, &mut Module>,
        args: ConstArgs,
    ) {
        let node_id = module.replace::<_, Const>(node_id, args);
        self.transform(module, node_id);
    }

    fn replace_with_multi_const(
        &mut self,
        node_id: NodeId,
        mut module: WithId<ModuleId, &mut Module>,
        args: impl IntoIterator<Item = ConstArgs>,
    ) {
        let node_id = module.replace::<_, MultiConst>(node_id, args);
        self.transform(module, node_id);
    }

    fn eliminate_const(
        &mut self,
        val: ConstVal,
        cons: Port,
        mut module: WithId<ModuleId, &mut Module>,
    ) {
        if self.netlist.cfg().no_eliminate_const || module.is_mod_output(cons) {
            return;
        }

        if let Some(&new_cons) = self.cons.get(&(module.id, val)) {
            module.reconnect_all_outgoing(cons.node, iter::once(new_cons));
        } else {
            self.cons.insert((module.id, val), cons);
        }
    }

    fn eliminate_multi_const(
        &mut self,
        node_id: NodeId,
        mut module: WithId<ModuleId, &mut Module>,
    ) {
        if self.netlist.cfg().no_eliminate_const {
            return;
        }

        let out_count = module[node_id].multi_cons().unwrap().out_count();
        for idx in 0 .. out_count {
            let val = module[node_id].multi_cons().unwrap().value(idx);

            self.eliminate_const(val, Port::new(node_id, idx as u32), module.reborrow());
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        netlist::NodeWithInputs,
        node::{Merger, MergerArgs, Splitter, SplitterArgs},
        node_ty::NodeTy,
        symbol::Symbol,
        visitor::reachability::Reachability,
    };

    fn transform(netlist: &NetList, mod_id: ModuleId) {
        Transform::new(netlist).visit_module(mod_id);

        let mut module = netlist[mod_id].borrow_mut();
        Reachability::new(netlist).visit_module(&mut module);
    }

    #[test]
    fn merger_splitter_pattern() {
        let mut module = Module::new("test", false);

        const IN1: u128 = 1;
        let input1_ty = NodeTy::Unsigned(IN1);
        let input1_sym = Some(Symbol::intern("input1"));
        let input1 = module.add_input(input1_ty, input1_sym);

        const IN2: u128 = 3;
        let input2_ty = NodeTy::Unsigned(IN2);
        let input2_sym = Some(Symbol::intern("input2"));
        let input2 = module.add_input(input2_ty, input2_sym);

        const IN3: u128 = 5;
        let input3_ty = NodeTy::Unsigned(IN3);
        let input3_sym = Some(Symbol::intern("input3"));
        let input3 = module.add_input(input3_ty, input3_sym);

        let merger = module.add_and_get_port::<_, Merger>(MergerArgs {
            inputs: [input1, input2, input3].into_iter(),
            rev: false,
            sym: Some(Symbol::intern("merger")),
        });

        let splitter = module.add::<_, Splitter>(SplitterArgs {
            input: merger,
            outputs: [input1_ty, input2_ty, input3_ty]
                .into_iter()
                .enumerate()
                .map(|(idx, ty)| {
                    let idx = idx + 1;
                    (
                        ty,
                        Some(Symbol::intern_args(format_args!("splitter_{idx}"))),
                    )
                }),
            start: None,
            rev: true,
        });

        module.add_mod_outputs(splitter);

        let mut netlist = NetList::default();
        let mod_id = netlist.add_module(module);

        transform(&netlist, mod_id);

        let pass1 = NodeWithInputs::pass(
            input1_ty,
            Some(Symbol::intern("splitter_1")),
            false,
            input1,
        );
        let pass2 = NodeWithInputs::pass(
            input2_ty,
            Some(Symbol::intern("splitter_2")),
            false,
            input2,
        );
        let pass3 = NodeWithInputs::pass(
            input3_ty,
            Some(Symbol::intern("splitter_3")),
            false,
            input3,
        );

        let module = netlist[mod_id].borrow();
        assert_eq!(module.nodes_vec(true), [
            NodeWithInputs::input(input1_ty, input1_sym, false),
            NodeWithInputs::input(input2_ty, input2_sym, false),
            NodeWithInputs::input(input3_ty, input3_sym, false),
            pass1.clone(),
            pass2.clone(),
            pass3.clone()
        ]);

        assert_eq!(module.mod_outputs_vec(true), [pass1, pass2, pass3]);
    }
}
