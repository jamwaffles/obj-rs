#[macro_use]
extern crate nom;

extern crate nalgebra;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str;

mod wavefront;

fn main() {
    let path = Path::new("./assets/cube.obj");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => ()
    }

	let obj = wavefront::parse_obj_string(&s).unwrap();

    let materials = match obj.mtllib {
        Some(ref filename) => {
            let pathname = format!("./assets/{}", filename);

            let path = Path::new(&pathname);
            let display = path.display();

            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why.description()),
                Ok(file) => file,
            };

            let mut mtlstring = String::new();

            match file.read_to_string(&mut mtlstring) {
                Err(why) => panic!("couldn't read {}: {}", display, why.description()),
                Ok(_) => ()
            }

            match wavefront::parse_mtl_string(&mtlstring) {
                Ok(materials) => Some(materials),
                Err(err) => {
                    println!("Material parse error: {}", err);

                    None
                }
            }
        },
        None => None
    };

	println!("{:?}", materials);
}