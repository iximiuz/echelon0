// pub struct Config {
//     sections: Vec<PluginSection>,
// }

// pub struct PluginSection {
// 	plugin_type: String,
// }

#[derive(Debug, PartialEq)]
pub struct Plugin {
    pub name: String,
}

impl Plugin {
    pub fn new(name: String) -> Plugin {
        Plugin { name: name }
    }
}
