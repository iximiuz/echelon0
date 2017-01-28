use pipeline::{InputSection, FilterSection, OutputSection};
use plugin::{InputPlugin, FilterPlugin, OutputPlugin};
use plugin::factory::PluginProvider;
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
            Err(_) => self.sess.errors.push("Cannot create input plugin"),
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
    use std::collections::HashSet;

    use config::ast::*;
    use plugin::factory::Result as PFResult;
    use plugin::factory::Error as PFError;
    use super::*;

    struct DummyFactory {
        inputs: HashSet<&'static str>,
        filters: HashSet<&'static str>,
        outputs: HashSet<&'static str>,
    }

    impl DummyFactory {
        pub fn new(inputs: Vec<&'static str>,
                   filters: Vec<&'static str>,
                   outputs: Vec<&'static str>)
                   -> DummyFactory {
            let mut factory = DummyFactory {
                inputs: HashSet::new(),
                filters: HashSet::new(),
                outputs: HashSet::new(),
            };
            for i in inputs {
                factory.inputs.insert(i);
            }
            for f in filters {
                factory.filters.insert(f);
            }
            for o in outputs {
                factory.outputs.insert(o);
            }
            factory
        }
    }

    impl PluginProvider for DummyFactory {
        fn create_input(&self, name: &str) -> PFResult<InputPlugin> {
            if self.inputs.contains(name) {
                Ok(InputPlugin {})
            } else {
                Err(PFError::PluginNotFound)
            }
        }
    }

    #[test]
    fn test_compile_simple() {
        let config = Config {
            sections: vec![
                PluginSection {
                    plugin_type: PluginType::Input,
                    block: vec![
                        BranchOrPlugin::Plugin(Plugin { name: "stdin".to_string() }),
                        BranchOrPlugin::Plugin(Plugin { name: "file".to_string() }),
                    ]
                },
                PluginSection { plugin_type: PluginType::Filter, block: vec![] },
                PluginSection {
                    plugin_type: PluginType::Output,
                    block: vec![
                        BranchOrPlugin::Plugin(Plugin { name: "stdout".to_string() }),
                        BranchOrPlugin::Plugin(Plugin { name: "file".to_string() }),
                    ],
                }
            ],
        };

        let mut factory = DummyFactory::new(vec!["stdin", "file"], vec![], vec!["stdout", "file"]);

        let sess = compile(&config, &factory);
        assert_eq!(sess.errors.len(), 0);
        assert_eq!(2, sess.inputs.count())
        // TODO: assert_eq!(0, sess.filters.count())
        // TODO: assert_eq!(2, sess.outputs.count())
    }
}
