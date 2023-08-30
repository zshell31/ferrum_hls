use crate::net_list::{ModuleId, NodeId, NodeOutId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Input,
    Output,
}

pub trait Visitor {
    fn visit_modules(&mut self);

    fn visit_module(&mut self, module_id: ModuleId);

    fn visit_param(&mut self, param: NodeOutId, kind: ParamKind);

    fn visit_node(&mut self, node_id: NodeId);
}
