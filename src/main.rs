#[macro_use]
extern crate nom;

extern crate nalgebra;

use nom::{ space, digit, line_ending, IResult, not_line_ending };
use nalgebra::{ Vector3 };
use std::str;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Face {
	vertices: [u32; 3],
	normals: [u32; 3],
}

#[derive(Debug)]
struct WavefrontObject {
	name: String,
	vertices: Vec<Vector3<f32>>,
	normals: Vec<Vector3<f32>>,
	texcoords: Option<Vec<Vector3<f32>>>,
	material: Option<String>,
	smoothing: Option<bool>,
	faces: Vec<Face>,
}

#[derive(Debug)]
struct WavefrontModel {
	mtllib: Option<String>,
	objects: Vec<WavefrontObject>,
}

named!(negative, tag!("-"));

named!(decimal, complete!(chain!(
    tag!(".")  ~
    val: digit ,
    || val
)));

named!(parse_float<f32>,
	do_parse!(
		sign: opt!(negative) >>
		bef: digit >>
		aft: opt!(decimal) >>
		({
			let a = match sign {
				Some(sign) => str::from_utf8(sign).unwrap(),
				None => ""
			};

			let b = str::from_utf8(bef).unwrap();

			let c = match aft {
				Some(aft) => str::from_utf8(aft).unwrap(),
				None => ""
			};

			format!("{}{}.{}", a, b, c).parse::<f32>().unwrap()
		})
	)
);

named!(parse_face_index<u32>,
	do_parse!(
		num: digit >>
		(str::from_utf8(num).unwrap().parse::<u32>().unwrap())
	)
);

named!(parse_vector3<&[u8], Vector3<f32>>,
	do_parse!(
		x: parse_float >>
		space >>
		y: parse_float >>
		space >>
		z: parse_float >>
		line_ending >>
		(Vector3::new(x, y, z))
	)
);

named!(vertex <&[u8], Vector3<f32>>, do_parse!(tag!("v") >> space >> vector: parse_vector3 >> (vector)));
named!(normal <&[u8], Vector3<f32>>, do_parse!(tag!("vn") >> space >> vector: parse_vector3 >> (vector)));
named!(texcoord<&[u8], Vector3<f32>>,
	do_parse!(
		tag!("vt") >>
		space >>
		u: parse_float >>
		space >>
		v: parse_float >>
		opt!(space) >>
		w: opt!(parse_float) >>
		({
			Vector3::new(u, v, w.unwrap_or(0.0))
		})
	)
);
named!(face <&[u8], Face>,
	do_parse!(
		tag!("f") >>
		space >>
		v1: parse_face_index >> tag!("/") >> vt1: opt!(parse_face_index) >> tag!("/") >> vn1: parse_face_index >>
		space >>
		v2: parse_face_index >> tag!("/") >> vt2: opt!(parse_face_index) >> tag!("/") >> vn2: parse_face_index >>
		space >>
		v3: parse_face_index >> tag!("/") >> vt3: opt!(parse_face_index) >> tag!("/") >> vn3: parse_face_index >>
		line_ending >>
		({
			Face {
				vertices: [ v1, v2, v3 ],
				normals: [ vn1, vn2, vn3 ],
			}
		})
	)
);

named!(vertices_aggregator<&[u8], Vec<Vector3<f32>>>, many1!(vertex));
named!(normals_aggregator<&[u8], Vec<Vector3<f32>>>, many1!(normal));
named!(texcoords_aggregator<&[u8], Vec<Vector3<f32>>>, many1!(texcoord));
named!(faces_aggregator<&[u8], Vec<Face>>, many0!(face));

named!(comment, preceded!(tag!("#"), take_until_and_consume!("\n")));

named!(mtllib<&[u8], String>,
	do_parse!(
		tag!("mtllib") >>
		space >>
		libname: not_line_ending >>
		line_ending >>
		(String::from(str::from_utf8(libname).unwrap_or("")))
	)
);

named!(usemtl<&[u8], String>,
	do_parse!(
		tag!("usemtl") >>
		space >>
		material: not_line_ending >>
		line_ending >>
		(String::from(str::from_utf8(material).unwrap_or("")))
	)
);

named!(smoothing<&[u8], bool>,
	do_parse!(
		tag!("s") >>
		space >>
		state: alt!(tag!("on") | tag!("off")) >>
		line_ending >>
		(match str::from_utf8(state).unwrap_or("").as_ref() {
			"off" => false,
			"on" => true,
			_ => false
		})
	)
);

named!(object_start<&[u8], String>,
	do_parse!(
		tag!("o") >>
		space >>
		name: take_until_and_consume!("\n") >>
		(String::from(str::from_utf8(name).unwrap_or("<invalid object name>")))
	)
);

named!(vertex_group<&[u8], WavefrontObject>,
	do_parse!(
		name: object_start >>
		vertices: vertices_aggregator >>
		normals: normals_aggregator >>
		textcoords: opt!(texcoords_aggregator) >>
		material: opt!(usemtl) >>
		smoothing: opt!(smoothing) >>
		faces: faces_aggregator >>
		({
			WavefrontObject {
				name: name,
				vertices: vertices,
				normals: normals,
				texcoords: textcoords,
				material: material,
				smoothing: smoothing,
				faces: faces,
			}
		})
	)
);

named!(obj_file<&[u8], WavefrontModel>,
	do_parse!(
		many0!(comment) >>
		mtllib: opt!(mtllib) >>
		objects: many1!(vertex_group) >>
		({
			WavefrontModel {
				mtllib: mtllib,
				objects: objects,
			}
		})
	)
);

fn parse_obj_file(input: &String) -> Result<WavefrontModel, String> {
	match obj_file(input.as_bytes()) {
		IResult::Done(_, object) => Ok(object),
		IResult::Incomplete(need) => {
			Err(format!("Incomplete, {:?}", need))
		},
		IResult::Error(err) => {
			Err(format!("Some error: {:?}", err))
		}
	}
}

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

	let obj = parse_obj_file(&s);

	println!("{:?}", obj);
}