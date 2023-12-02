use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::{
    net_list::{ModuleId, NetList, NodeId},
    node::{IsNode, NodeKind},
    params::Outputs,
    symbol::Symbol,
    visitor::Visitor,
};

static DEFAULT_SYMBOLS: Lazy<FnvHashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("input", "input$"),
        ("output", "output$"),
        ("reg", "reg$"),
        ("self", "self$"),
    ]
    .into_iter()
    .map(|(sym1, sym2)| (sym1, sym2))
    .collect()
});

pub struct SetNames<'n> {
    net_list: &'n mut NetList,
    idents: FnvHashMap<(ModuleId, Symbol), usize>,
    module_idents: FnvHashMap<Symbol, usize>,
}

impl<'n> SetNames<'n> {
    pub fn new(net_list: &'n mut NetList) -> Self {
        Self {
            net_list,
            idents: Default::default(),
            module_idents: Default::default(),
        }
    }

    pub fn run(&mut self) {
        self.visit_modules();
    }

    fn ident(sym: Symbol, count: Option<usize>) -> (Symbol, usize) {
        let sym = DEFAULT_SYMBOLS
            .get(sym.as_str())
            .map(|new_sym| Symbol::new(new_sym))
            .unwrap_or(sym);

        match count {
            Some(mut count) => {
                count += 1;
                (sym, count)
            }
            None => (sym, 0),
        }
    }

    fn make_sym(sym: Symbol, count: usize) -> Symbol {
        if count == 0 {
            sym
        } else {
            Symbol::new_from_args(format_args!("{}_{}", sym, count))
        }
    }

    fn set_module_name(&mut self, module_id: ModuleId) {
        let module = &mut self.net_list[module_id];
        let sym = module.name;
        let count = self.module_idents.get(&sym).copied();

        let (new_sym, count) = Self::ident(sym, count);
        self.module_idents.insert(sym, count);
        module.name = Self::make_sym(new_sym, count);
    }

    fn set_node_out_names(&mut self, node_id: NodeId) {
        let module_id = node_id.module_id();
        let node = &mut self.net_list[node_id];

        for out in node.kind.outputs_mut().items_mut() {
            let sym = out.out.sym.unwrap_or(Symbol::new("__tmp"));
            let count = self.idents.get(&(module_id, sym)).copied();

            let (new_sym, count) = Self::ident(sym, count);
            self.idents.insert((module_id, sym), count);
            out.out.sym = Some(Self::make_sym(new_sym, count));
        }

        let mut new_name = None;

        if let NodeKind::ModInst(mod_inst) = &self.net_list[node_id].kind {
            let sym = Symbol::new_from_args(format_args!(
                "__{}",
                self.net_list[mod_inst.module_id].name
            ));
            if mod_inst.name.is_none() {
                let count = self.idents.get(&(module_id, sym)).copied();

                let (new_sym, count) = Self::ident(sym, count);
                self.idents.insert((module_id, sym), count);
                new_name = Some(Self::make_sym(new_sym, count))
            }
        }

        if let NodeKind::ModInst(mod_inst) = &mut self.net_list[node_id].kind {
            mod_inst.name = mod_inst.name.or(new_name);
        }
    }
}

impl<'n> Visitor for SetNames<'n> {
    fn visit_modules(&mut self) {
        for module_id in self.net_list.modules() {
            let module = &self.net_list[module_id];
            if module.is_skip {
                continue;
            }
            self.visit_module(module_id);
        }
    }

    fn visit_module(&mut self, module_id: ModuleId) {
        self.set_module_name(module_id);

        let mut cursor = self.net_list.mod_cursor(module_id);
        while let Some(node_id) = self.net_list.next(&mut cursor) {
            let node = &self.net_list[node_id];
            if node.is_skip {
                continue;
            }

            self.visit_node(node_id);
        }
    }

    fn visit_node(&mut self, node_id: NodeId) {
        self.set_node_out_names(node_id);
    }
}