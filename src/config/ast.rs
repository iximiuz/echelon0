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

#[derive(Debug, PartialEq)]
pub enum BoolOperator {
    And,
    Or,
    Xor,
    Nand,
}

#[derive(Debug, PartialEq)]
pub enum CompareOperator {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Leaf(Box<BoolExpr>),
    Branch(BoolOperator, Box<Condition>, Box<Condition>),
}

#[derive(Debug, PartialEq)]
pub enum BoolExpr {
    Parens(Box<Condition>),
    Compare(CompareOperator, Box<Rvalue>, Box<Rvalue>),
    Rvalue(Box<Rvalue>),
}

#[derive(Debug, PartialEq)]
pub enum Rvalue {
    String(String),
    Number(f64),
}
