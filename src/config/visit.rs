use config::ast::*;

pub trait Visitor<'ast>: Sized {
    fn visit_section(&mut self, section: &'ast PluginSection) {
        walk_plugin_section(self, section)
    }

    fn visit_input_block(&mut self, block: &'ast Block) {
        walk_block!("input", self, block)
    }

    fn visit_filter_block(&mut self, section: &'ast Block) {
        walk_block!("filter", self, block)
    }

    fn visit_output_block(&mut self, section: &'ast Block) {
        walk_block!("output", self, block)
    }

    fn visit_input_plugin(&mut self, plugin: &'ast Plugin) {

    }

    fn visit_input_branch(&mut self, branch: &'ast Branch) {

    }

    fn visit_filter_plugin(&mut self, plugin: &'ast Plugin) {

    }

    fn visit_filter_branch(&mut self, branch: &'ast Branch) {

    }

    fn visit_output_plugin(&mut self, plugin: &'ast Plugin) {

    }

    fn visit_output_branch(&mut self, branch: &'ast Branch) {

    }
}

/// Entry point of the AST visiting.
pub fn walk_config<'a, V: Visitor<'a>>(visitor: &mut V, config: &'a Config) {
    for section in &config.sections {
        visitor.visit_section(section);
    }
}

pub fn walk_plugin_section<'a, V: Visitor<'a>>(visitor: &mut V, section: &'a PluginSection) {
    match section.plugin_type {
        PluginType::Input => visitor.visit_input_block(&section.block),
        PluginType::Filter => visitor.visit_filter_block(&section.block),
        PluginType::Output => visitor.visit_output_block(&section.block),
    }
}

pub fn walk_block<'a, V: Visitor<'a>>(visitor: &mut V, block: &'a Block) {
    for branch_or_plugin in block {
        match branch_or_plugin {
            BranchOrPlugin::Plugin(p) => visitor.visit_plugin(&p),
            BranchOrPlugin::Branch(b) => visitor.visit_plugin(&b),
        }
    }
}
