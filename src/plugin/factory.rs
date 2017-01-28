use super::input::InputPlugin;

pub enum Error {
    PluginNotFound,
}

// TODO: impl From and std::error::Error for Error

pub type Result<T> = ::std::result::Result<T, Error>;

pub trait PluginProvider {
    fn create_input(&self, name: &str) -> Result<InputPlugin>;
}

pub struct PluginFactory {

}

impl PluginFactory {
    pub fn new() -> PluginFactory {
        PluginFactory {}
    }
}

impl PluginProvider for PluginFactory {
    fn create_input(&self, name: &str) -> Result<InputPlugin> {
        Ok(InputPlugin {})
    }
}
