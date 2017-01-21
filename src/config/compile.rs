use super::ast::*;
use super::visit;
use super::visit::Visitor;
use pipeline::{InputSection, FilterSection, OutputSection};
use plugin::PluginFactory;

struct Compiler {
    plugin_factory: PluginFactory,
    sess: Session,
}

impl<'ast> Visitor<'ast> for Compiler {
    fn visit_input_plugin(&mut self, plugin: &'ast Plugin) {
        self.sess.inputs.add_plugin(self.plugin_factory.create_input(&plugin.name))
    }

    fn visit_input_branch(&mut self, _: &'ast Branch) {
        self.sess.errors.push("Conditional inputs are forbidden")
    }
}

pub struct Session {
    pub errors: Vec<&'static str>,
    pub inputs: InputSection,
    pub filters: FilterSection,
    pub outputs: OutputSection,
}

pub fn compile(config: &Config, plugin_factory: PluginFactory) -> Session {
    let sess = Session {
        errors: vec![],
        inputs: InputSection::new(),
        filters: FilterSection::new(),
        outputs: OutputSection::new(),
    };
    let mut compiler = Compiler {
        plugin_factory: plugin_factory,
        sess: sess
    };

    visit::walk_config(&mut compiler, config);
    compiler.sess
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let sess = compile(&Config { sections: vec![] }, PluginFactory::new());
    }
}
