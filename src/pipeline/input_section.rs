use plugin::InputPlugin;

pub struct InputSection {
    inputs: Vec<InputPlugin>,
}

impl InputSection {
    pub fn new() -> InputSection {
        InputSection { inputs: vec![] }
    }

    pub fn add_plugin(&mut self, input: InputPlugin) {
        self.inputs.push(input);
    }
}
