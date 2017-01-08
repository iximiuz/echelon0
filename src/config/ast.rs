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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoolOperator {
    And,
    Or,
}

impl BoolOperator {
    pub fn precedence(&self) -> i32 {
        match *self {
            BoolOperator::Or => 100,
            BoolOperator::And => 200,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompareOperator {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl CompareOperator {
    pub fn to_string(&self) -> &'static str {
        use self::CompareOperator::*;
        match *self {
            Eq => "==",
            Ne => "!=",
            Lt => "<",
            Gt => ">",
            Le => "<=",
            Ge => ">=",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Leaf(Box<BoolExpr>),
    Branch(BoolOperator, Box<Condition>, Box<Condition>),
}

impl From<BoolExpr> for Condition {
    fn from(v: BoolExpr) -> Condition {
        Condition::Leaf(Box::new(v))
    }
}

#[derive(Debug, PartialEq)]
pub enum BoolExpr {
    Parens(Box<Condition>),
    Negative(Box<BoolExpr>), // TODO: maybe use Condition instead of BoolExpr here?
    Compare(CompareOperator, Box<Rvalue>, Box<Rvalue>),
    Rvalue(Box<Rvalue>),
}

impl BoolExpr {
    pub fn not(self) -> BoolExpr {
        BoolExpr::Negative(Box::new(self))
    }
}

impl From<Rvalue> for BoolExpr {
    fn from(v: Rvalue) -> BoolExpr {
        BoolExpr::Rvalue(Box::new(v))
    }
}

#[derive(Debug, PartialEq)]
pub struct Selector {
    elements: Vec<String>,
}

impl Selector {
    pub fn new(elements: Vec<String>) -> Selector {
        Selector { elements: elements }
    }
}

#[derive(Debug, PartialEq)]
pub enum Rvalue {
    Number(f64),
    String(String),
    Selector(Selector),
}

impl From<f64> for Rvalue {
    fn from(v: f64) -> Self {
        Rvalue::Number(v)
    }
}

impl From<String> for Rvalue {
    fn from(v: String) -> Self {
        Rvalue::String(v)
    }
}

impl From<&'static str> for Rvalue {
    fn from(v: &'static str) -> Self {
        Rvalue::String(v.to_string())
    }
}

impl From<Selector> for Rvalue {
    fn from(v: Selector) -> Self {
        Rvalue::Selector(v)
    }
}
