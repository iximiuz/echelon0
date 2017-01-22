use super::input::InputPlugin;

pub enum Error {
    PluginNotFound
}

pub trait PluginProvider {
    fn create_input(&self, name: &str) -> Result<InputPlugin, Error>;
}

pub struct PluginFactory {

}

impl PluginFactory {
    pub fn new() -> PluginFactory {
        PluginFactory {}
    }
}

impl PluginProvider for PluginFactory {
    fn create_input(&self, name: &str) -> Result<InputPlugin, Error> {
        Ok(InputPlugin {})
    }
}
