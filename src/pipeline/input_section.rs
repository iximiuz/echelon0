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

        self.inputs.append(&mut extra_inputs);
        while let Some(mut input) = self.inputs.pop() {
            input.register();
            // TODO: set thread name to "[#{pipeline_id}]<#{plugin.class.config_name}"
            self.workers.push(thread::Builder::new()
                .name("[pipeline_id]<input_name".to_string())
                .spawn(move || {
                    let mut w = InputWorker { input: input };
                    w.run();
                }).expect("Cannot start Input worker"));
        }
    }

    pub fn wait(&mut self) {
        while let Some(w) = self.workers.pop() {
            match w.join() {
                Err(e) => {
                    // TODO: log
                }
                _ => {},
            }
        }
    }
}

// TODO: impl Drop for InputSection

struct InputWorker {
    input: InputPlugin,
}

impl InputWorker {
    pub fn run(&mut self) {
        self.input.run();
    }
}
