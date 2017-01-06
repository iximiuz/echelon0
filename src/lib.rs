#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate regex;
// extern crate serde;
// extern crate serde_json;

mod macros;
mod config;

pub struct Runner {

}

impl Runner {
    pub fn new() -> Runner {
        Runner {}
    }

    pub fn run(&self) {
        println!("Hello from Echelon0 runner!");
    }
}
