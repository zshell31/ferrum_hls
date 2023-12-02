use crate::{
    net_list::{ModuleId, NetList, NodeId},
    node::IsNode,
    visitor::Visitor,
};

pub struct Dump<'n> {
    net_list: &'n NetList,
    skip: bool,
}

impl<'n> Dump<'n> {
    pub fn new(net_list: &'n NetList, skip: bool) -> Self {
        Self { net_list, skip }
    }

    pub fn run(&mut self) {
        self.visit_modules();
    }
}

impl<'n> Visitor for Dump<'n> {
    fn visit_modules(&mut self) {
        for module_id in self.net_list.modules() {
            let module = &self.net_list[module_id];
            if self.skip && module.is_skip {
                continue;
            }
            module.dump();
            self.visit_module(module_id);
        }

        println!("\n");
    }

    fn visit_module(&mut self, module_id: ModuleId) {
        let tab: &'static str = "        ";
        let mut cursor = self.net_list.mod_cursor(module_id);
        while let Some(node_id) = self.net_list.next(&mut cursor) {
            let node = &self.net_list[node_id];

            if self.skip && node.is_skip {
                continue;
            }

            let prefix = format!("{:>4}    ", node_id.idx().unwrap());

            node.dump(&prefix, tab);

            println!("\n{}links:", tab);
            for out in node.kind.node_out_ids(node_id) {
                if self.net_list.links(out).next().is_some() {
                    println!(
                        "{}{} -> {}",
                        tab,
                        out.out_id(),
                        self.net_list
                            .links(out)
                            .map(|(node_id, _)| node_id.idx().unwrap().to_string())
                            .intersperse(", ".to_string())
                            .collect::<String>()
                    );
                }
            }
            println!();
        }
    }

    fn visit_node(&mut self, _node_id: NodeId) {}
}