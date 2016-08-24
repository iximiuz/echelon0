use std::io;
use std::io::{Read, BufRead, BufReader, Stdin};
use std::fs::File;

extern crate multi_reader;


pub struct Input<R> {
    source: R,
}

impl<R: BufRead> Input<R> {
    // pub fn read_chunk(&mut self, u8, buf: &mut Vec<u8>) -> Result<(), String> {
    // let f = File::open("input.txt").expect("File not found");
    //    Ok(())
    // }
}

impl<I: Iterator<Item = File>> Input<multi_reader::MultiReader<File, I>> {
    pub fn files(files: I) -> Input<BufReader<multi_reader::MultiReader<File, I>>> {
        Input { source: BufReader::new(multi_reader::MultiReader::new(files)) }
    }
}

impl<'a> Input<io::StdinLock<'a>> {
    pub fn stdin(stdin: &'a Stdin) -> Input<io::StdinLock<'a>> {
        Input { source: stdin.lock() }
    }
}
