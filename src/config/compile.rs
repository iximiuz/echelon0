use pipeline::{InputSection, FilterSection, OutputSection};
use plugin::{InputPlugin, PluginProvider, Error};
use super::ast::*;
use super::visit;
use super::visit::Visitor;

struct Compiler<'a> {
    plugin_provider: &'a PluginProvider,
    sess: Session,
}

impl<'a, 'ast> Visitor<'ast> for Compiler<'a> {
    fn visit_input_plugin(&mut self, plugin: &'ast Plugin) {
        match self.plugin_provider.create_input(&plugin.name) {
            Ok(p) => self.sess.inputs.add_plugin(p),
            Err(_) => self.sess.errors.push("Cannot create input plugin")
        }
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

pub fn compile(config: &Config, plugin_provider: &PluginProvider) -> Session {
    let sess = Session {
        errors: vec![],
        inputs: InputSection::new(),
        filters: FilterSection::new(),
        outputs: OutputSection::new(),
    };
    let mut compiler = Compiler {
        plugin_provider: plugin_provider,
        sess: sess,
    };

    visit::walk_config(&mut compiler, config);
    compiler.sess
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct DummyFactory {
        pub inputs: HashMap<&'static str, InputPlugin>,
    }

    impl PluginProvider for DummyFactory {
        fn create_input(&self, name: &str) -> Result<InputPlugin, Error> {
            match self.inputs.get(name) {
                Some(p) => Ok(InputPlugin {}),
                None => Err(Error::PluginNotFound)
            }
        }
    }

    #[test]
    fn test_compile_simple() {
        let mut factory = DummyFactory { inputs: HashMap::new() };
        factory.inputs.insert("stdin", InputPlugin {});

        let sess = compile(&Config { sections: vec![] }, &factory);
        assert_eq!(sess.errors.len(), 0);

        // TODO: ...
    }
}
