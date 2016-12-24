// pub struct Config {
//     sections: Vec<PluginSection>,
// }

// pub struct PluginSection {
// 	plugin_type: String,
// }

pub enum PluginType {
    Input,
    Filter,
    Output,
}

#[derive(Debug, PartialEq)]
pub struct Plugin {
    pub name: String,
}

impl Plugin {
    pub fn new(name: String) -> Plugin {
        Plugin { name: name }
    }
}

pub struct Branch {

}

pub enum BranchOrPlugin {
    Branch(Branch),
    Plugin(Plugin),
}

pub struct Condition {

}
