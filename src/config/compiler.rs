use config::ast;
use pipeline::{InputSection, FilterSection, OutputSection};
use plugin::{InputPlugin, FilterPlugin, OutputPlugin, PluginFactory};

pub struct Compiler {
    config_ast: ast::Config,
    plugin_factory: PluginFactory,
}

impl Compiler {
    pub fn new(config_ast: ast::Config, plugin_factory: PluginFactory) -> Compiler {
        Compiler {
            config_ast: config_ast,
            plugin_factory: plugin_factory,
        }
    }

    pub fn generate_inputs(&self) -> InputSection {
        InputSection {}
    }

    pub fn generate_filters(&self) -> FilterSection {
        FilterSection {}
    }

    pub fn generate_outputs(&self) -> OutputSection {
        OutputSection {}
    }
}
