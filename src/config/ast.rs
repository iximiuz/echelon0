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

pub enum BranchOrPlugin {
    Branch(Branch),
    Plugin(Plugin),
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

/// A branch is essentially a vec of cases `if {...} else if {...} else if {...} else {...}`.
///
/// I.e. cases[0] is always `if` statement. And `else` is generalized as `else if (true)` and
/// always goes as the last one vec element (if exists).
pub struct Branch {
    cases: Vec<Case>,
}

impl Branch {
    pub fn new(case_if: Case, else_ifs: &mut Vec<Case>, case_else: Option<Case>) -> Branch {
        let cases = Vec::with_capacity(else_ifs.len() + 2);
        cases.push(case_if);
        cases.append(else_ifs);
        if let Some(c) = case_else {
            cases.push(c);
        }
        Branch { cases: cases }
    }
}

/// Conditional block.
///
/// I.e. a condition from `if` or `else if` or `else` statement plus a block.
pub struct Case {
    condition: Condition,
    block: Block,
}

impl Case {
    pub fn new(condition: Condition, block: Block) -> Case {
        Case {
            condition: condition,
            block: block,
        }
    }
}

pub type Block = Vec<BranchOrPlugin>;

#[derive(Debug, PartialEq)]
pub enum Condition {
    Leaf(Box<BoolExpr>),
    Branch(BoolOperator, Box<Condition>, Box<Condition>),
}

impl Condition {
    pub fn truth() -> Condition {
        Condition::Leaf(Box::new(BoolExpr::from(Rvalue::from(1.0))))
    }
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
