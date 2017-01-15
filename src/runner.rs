use config::compiler::Compiler;
use config::parser;
use pipeline::Pipeline;
use plugin::PluginFactory;

/// Handles program runs (i.e. parses command line params and dispatches executors).
pub struct Runner {
    pipeline: Pipeline,
}

impl Runner {
    pub fn new() -> Runner {
        let compiler = Compiler::new(
            parser::parse(include_bytes!("./config/tests/assets/simplest.conf")).unwrap(),
            PluginFactory::new(),
        );
        let pipeline = Pipeline::new(
            compiler.generate_inputs(),
            compiler.generate_filters(),
            compiler.generate_outputs(),
        );
        Runner { pipeline: pipeline }
    }

    pub fn run(&self) {
        println!("Hello from Echelon0 runner!");
    }
}
