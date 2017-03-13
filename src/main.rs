#[macro_use]
extern crate nom;

extern crate nalgebra;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod wavefront;

// fn parse_obj_file(input: &String) -> Result<WavefrontModel, String> {
// 	match obj_file(input.as_bytes()) {
// 		IResult::Done(_, object) => Ok(object),
// 		IResult::Incomplete(need) => {
// 			Err(format!("Incomplete, {:?}", need))
// 		},
// 		IResult::Error(err) => {
// 			Err(format!("Some error: {:?}", err))
// 		}
// 	}
// }

fn main() {
	// Create a path to the desired file
    let path = Path::new("./assets/cube.obj");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => ()
    }

	let obj = wavefront::parse_obj_string(&s);

	println!("{:?}", obj);
}