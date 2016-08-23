use std::io;
use std::io::{BufReader, Read, Stdin};
use std::fs::File;


pub struct Input<R> {
    source: BufReader<R>,
}

impl<R: Read> Input<R> {
    pub fn stdin<'a>(stdin: &'a Stdin) -> Input<Stdin> {
        Input { source: stdin.lock() }
    }

    pub fn files<I: Iterator<Item = File>>(files: I) -> Input<ChainedReader<File, I>> {
        Input { source: BufReader::new(ChainedReader::new(files)) }
    }

    // pub fn read_chunk(&mut self, u8, buf: &mut Vec<u8>) -> Result<(), String> {
    // let f = File::open("input.txt").expect("File not found");
    //    Ok(())
    // }
}

pub struct ChainedReader<R, I> {
    readers: I,
    current: Option<R>,
}

impl<R: Read, I: Iterator<Item = R>> ChainedReader<R, I> {
    pub fn new(mut readers: I) -> ChainedReader<R, I> {
        let current = readers.next();
        ChainedReader {
            readers: readers,
            current: current,
        }
    }
}

impl<R: Read, I: Iterator<Item = R>> Read for ChainedReader<R, I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.current {
                Some(ref mut r) => {
                    let n = try!(r.read(buf));
                    if n > 0 {
                        return Ok(n);
                    }
                }
                None => return Ok(0),
            }
            self.current = self.readers.next();
        }
    }
}
