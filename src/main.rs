use std::env;
use std::io::BufRead;
use std::process;

use getopts::Options;

extern crate env_logger;
extern crate getopts;

extern crate monstrio;

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

    // let stdin = io::stdin();
    // let stdin_input = monstrio::Input::stdin(&stdin);

    let mut glob_input = monstrio::Input::glob(args.free.into_iter());
    let reader = glob_input.as_mut();
    let parser = match parser::Parser::new(r"/(.+)/ line") { // args.opt_str("i"), args.opt_str("e")
        Ok(p) => p,
        Err(err) => {
            println!("Cannot create parser! {:?}", err);
            return;
        }
    };

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }

        match parser.parse_entry(&line) {
            Ok(entry) => {
                println!("{:?}", entry);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}
