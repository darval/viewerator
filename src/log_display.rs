use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;

pub struct LogDisplay {
    fd: File,
}

impl LogDisplay {
    pub fn new() -> LogDisplay {
        LogDisplay {
            fd: File::open("/var/log/minerator.log").unwrap(),
        }
    }

    pub fn init() {}

    pub fn read_raw(&mut self) -> Vec<String> {
        let mut raw = Vec::new();
        let mut reader = BufReader::new(&self.fd);
        let length = reader.seek(SeekFrom::End(0)).unwrap();
        if length > 80000 {
            reader.seek(SeekFrom::End(-80000)).unwrap();
        } else {
            reader.seek(SeekFrom::Start(0)).unwrap();
        }
        for line in reader.lines().map(|l| l.unwrap()) {
            raw.push(line);
        }
        raw
    }
}
