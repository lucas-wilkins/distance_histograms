

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read, ErrorKind};
use std::error::Error;
use std::cmp::min;

pub struct XYZData {
    data: Vec<f64>,
    n_points: usize
}

impl XYZData {
    fn new(data: Vec<f64>) -> Self {

        if data.len() % 3 != 0 {
            panic!("Wrong number of entries in file");
        }

        let n_points = data.len() / 3;

        Self {
            data: data,
            n_points: n_points 
        }
    }

    pub fn show_some_data(&self) {
        /* Show the start of loaded xyz data */
    
        let max_index: usize = min(21, self.data.len());
    
        for index in 0 .. max_index {
            if index % 3 == 2 {
                println!("{}", self.data[index]);
            } else {
                print!("{}, ", self.data[index]);
            }
        }
    }
}

pub fn load_data(file_path: &PathBuf, scaling: f64) -> XYZData {
    
    let file: File = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file, {}", e)
    };

    let mut reader: BufReader<File> = BufReader::new(file);

    let mut buffer: [u8; 8] = [0u8; 8];
    
    let mut raw_data: Vec<f64> = Vec::new();

    loop {
        if let Err(e) = reader.read_exact(&mut buffer) {
            if e.kind() == ErrorKind::UnexpectedEof {
                break;
            }

            panic!("Error reading file {}", e);
        }

        raw_data.push(f64::from_le_bytes(buffer) * scaling);
    

    }


    XYZData::new(raw_data)

}


