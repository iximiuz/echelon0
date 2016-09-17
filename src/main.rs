use std::env;
use std::io;
use std::io::BufRead;
use std::process;

use getopts::Options;

extern crate env_logger;
extern crate getopts;

extern crate monstrio;

mod echelon0;
mod macros;
mod parser;
mod rule;


fn print_usage(opts: &Options, program: &String) {
    let brief = format!("Usage: {} parse_rule [glob ...] [-i include_pattern | -e \
                         exclude_pattern]",
                        program);
    println!("{}", opts.usage(&brief));
}

fn handle_bad_opts(err: &String, program: &String) {
    println!("{} Try \"{} -h\" for help.", err, program);
    process::exit(1);
}

fn main() {
    env_logger::init().expect("Cannot initialize logger");

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i",
                "include",
                "if presents only entries matched to this pattern will be parsed",
                "INCLUDE");
    opts.optopt("e",
                "exclude",
                "if presents only entries not matched to this pattern will be processed",
                "EXCLUDE");
    opts.optflag("h", "help", "show this message");
    let args = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return handle_bad_opts(&f.to_string(), &program),
    };

    if args.opt_present("h") {
        print_usage(&opts, &program);
        return;
    }

    let parser = match parser::Parser::new(&args.free[0]) { 
        Ok(p) => p,
        Err(err) => {
            println!("Cannot create parser! {:?}", err);
            return;
        }
    };

    let stdin;
    let mut stdin_input;
    let mut glob_input;
    let reader: &mut BufRead = if args.free.len() > 1 {
        glob_input = monstrio::Input::glob(args.free[1..].into_iter());
        glob_input.as_mut()
    } else {
        stdin = io::stdin();
        stdin_input = monstrio::Input::stdin(&stdin);
        stdin_input.as_mut()
    };

    let mut echelon0 = echelon0::Echelon0::new(reader, &parser);
    if let Some(ref include) = args.opt_str("i") {
        echelon0.set_include_filter(include);
    }
    if let Some(ref exclude) = args.opt_str("e") {
        echelon0.set_exclude_filter(exclude);
    }

    echelon0.start();
}
