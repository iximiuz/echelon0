use super::ast;
use super::visit;
use super::visit::Visitor;
use pipeline::{InputSection, FilterSection, OutputSection};
use plugin::PluginFactory;

struct Compiler {
    plugin_factory: PluginFactory,
    sess: Session,
}

impl<'ast> Visitor<'ast> for Compiler {
}

pub struct Session {
    pub errors: Vec<String>,
    pub inputs: InputSection,
    pub filters: FilterSection,
    pub outputs: OutputSection,
}

pub fn compile(config: &ast::Config, plugin_factory: PluginFactory) -> Session {
    let sess = Session {
        errors: vec![],
        inputs: InputSection {},
        filters: FilterSection {},
        outputs: OutputSection {},
    };
    let mut compiler = Compiler {
        plugin_factory: plugin_factory,
        sess: sess
    };

    visit::walk_config(&mut compiler, config);
    compiler.sess
}