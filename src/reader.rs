use std::fs::File;
use std::path::Path;

extern crate glob;
use self::glob::glob;


// Facade
pub struct Reader {
    // owns actual reader
}

impl Reader {
    pub fn from_std_in() -> Reader {
        Reader {}
    }

    pub fn from_files<P: AsRef<Path>>(paths: &[P]) -> Reader {
        Reader {}
    }

    pub fn read_chunk(to: &mut String) -> Result<(), String> {

        // let f = File::open("input.txt").expect("File not found");
        Ok(())
    }
}

// Also can be used to read from StdIO
struct FileReader {
}

struct CompositeReader {
}
