use super::{InputSection, FilterSection, OutputSection};

pub struct Pipeline {
    inputs: InputSection,
    filters: FilterSection,
    outputs: OutputSection,
}

impl Pipeline {
    pub fn new(inputs: InputSection, filters: FilterSection, outputs: OutputSection) -> Pipeline {
        Pipeline {
            inputs: inputs,
            filters: filters,
            outputs: outputs,
        }
    }

    pub fn run(&mut self) {
        // Starts all the extra threads and waits them too.
        self.start_workers();
        self.inputs.wait();
    }

    fn start_workers(&mut self) {
        self.inputs.run();
    }
}
