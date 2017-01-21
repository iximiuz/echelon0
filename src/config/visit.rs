use config::ast::*;

pub trait Visitor<'ast>: Sized {
    fn visit_section(&mut self, section: &'ast PluginSection) {
        walk_plugin_section(self, section)
    }

    fn visit_input_block(&mut self, block: &'ast Block) {
        walk_input_block(self, block)
    }

    fn visit_filter_block(&mut self, block: &'ast Block) {
        walk_filter_block(self, block)
    }

    fn visit_output_block(&mut self, block: &'ast Block) {
        walk_output_block(self, block)
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

macro_rules! walk_block {
    ($visitor: expr, $visit_plugin_method: ident, $visit_branch_method: ident, $block: expr) => {
        for branch_or_plugin in $block {
            match *branch_or_plugin {
                BranchOrPlugin::Plugin(ref p) => $visitor.$visit_plugin_method(p),
                BranchOrPlugin::Branch(ref b) => $visitor.$visit_branch_method(b),
            }
        }
    };
}

pub fn walk_input_block<'a, V: Visitor<'a>>(visitor: &mut V, block: &'a Block) {
    walk_block!(visitor, visit_input_plugin, visit_input_branch, block)
}

pub fn walk_filter_block<'a, V: Visitor<'a>>(visitor: &mut V, block: &'a Block) {
    walk_block!(visitor, visit_filter_plugin, visit_filter_branch, block)
}

pub fn walk_output_block<'a, V: Visitor<'a>>(visitor: &mut V, block: &'a Block) {
    walk_block!(visitor, visit_output_plugin, visit_output_branch, block)
}
