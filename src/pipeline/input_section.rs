use std::thread;

use plugin::InputPlugin;

pub struct InputSection {
    inputs: Vec<InputPlugin>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl InputSection {
    pub fn new() -> InputSection {
        InputSection {
            inputs: vec![],
            workers: vec![],
        }
    }

    pub fn add_plugin(&mut self, input: InputPlugin) {
        self.inputs.push(input);
    }

    pub fn run(&mut self) {
        let mut extra_inputs = Vec::new();
        for input in &self.inputs {
            for _k in 1..input.threads_count() {
                extra_inputs.push(input.clone());
            }
        }

        for input in &mut self.inputs {
            input.register();
            // TODO: set thread name to "[#{pipeline_id}]<#{plugin.class.config_name}"
            self.workers.push(thread::Builder::new()
                .name("[pipeline_id]<input_name".to_string())
                .spawn(move || {
                    let mut w = InputWorker { input: input };
                    w.run();
                }).expect("Cannot start Input worker"));
        }
        for input in &mut extra_inputs {
            input.register();
        }
    }

    pub fn wait(&self) {
        for w in self.workers {
            w.join();
        }
    }
}

struct InputWorker<'a> {
    input: &'a InputPlugin,
}

impl<'a> InputWorker<'a> {
    pub fn run(&mut self) {
        self.input.run();
    }
}
