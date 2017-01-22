use config::compile::compile;
use config::parse::parse;
use pipeline::Pipeline;
use plugin::PluginFactory;

/// Handles program runs (i.e. parses command line params and dispatches executors).
pub struct Runner {
    pipeline: Pipeline,
}

impl Runner {
    pub fn new() -> Runner {
        let config = parse(include_bytes!("./config/tests/assets/simplest.conf")).unwrap();
        let plugin_factory = PluginFactory::new();
        let session = compile(&config, &plugin_factory);
        let pipeline = Pipeline::new(session.inputs, session.filters, session.outputs);
        Runner { pipeline: pipeline }
    }

    pub fn run(&self) {
        println!("Hello from Echelon0 runner!");
    }
}
