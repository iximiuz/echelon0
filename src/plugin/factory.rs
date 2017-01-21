use super::input::InputPlugin;

pub struct PluginFactory {

}

impl PluginFactory {
    pub fn new() -> PluginFactory {
        PluginFactory {}
    }

    pub fn create_input(&self, name: &str) -> InputPlugin {
        InputPlugin {}
    }
}
