use nom::{ space, digit, line_ending, IResult, not_line_ending };
use nalgebra::{ Vector3 };
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

use super::mtl;

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
pub struct WavefrontModel {
	materials: Option<mtl::WavefrontMaterials>,
	objects: Vec<WavefrontObject>,
}

#[derive(Copy, Clone)]
pub struct BufferVertex {
    position: [ f32; 3 ],
	normal: [ f32; 3 ],
}

implement_vertex!(BufferVertex, position, normal);

impl WavefrontModel {
	pub fn to_vertices(&self) -> (Vec<BufferVertex>, mtl::WavefrontMaterial) {
		let object = self.objects.get(0).unwrap();

		let vertices = object.faces.iter().flat_map(|f| {
			let v1 = object.vertices.get(f.vertices[0] as usize).unwrap();
			let v2 = object.vertices.get(f.vertices[1] as usize).unwrap();
			let v3 = object.vertices.get(f.vertices[2] as usize).unwrap();

			let vn1 = object.normals.get(f.normals[0] as usize).unwrap();
			let vn2 = object.normals.get(f.normals[1] as usize).unwrap();
			let vn3 = object.normals.get(f.normals[2] as usize).unwrap();

			vec![
				BufferVertex {
					position: [ v1.x, v1.y, v1.z ],
					normal: [ vn1.x, vn1.y, vn1.z ],
					// color: diffuse,
				},

				BufferVertex {
					position: [ v2.x, v2.y, v2.z ],
					normal: [ vn2.x, vn2.y, vn2.z ],
					// color: diffuse,
				},

				BufferVertex {
					position: [ v3.x, v3.y, v3.z ],
					normal: [ vn3.x, vn3.y, vn3.z ],
					// color: diffuse,
				},
			]
		}).collect();

		let material = match &self.materials {
			&Some(ref materials) => {
				match &materials.get("Material") {
					&Some(ref mat) => (*mat).clone(),
					&None => mtl::WavefrontMaterial {
						name: String::from("Default material"),
						specular_exponent: 1.0,
						ambient: [0.1, 0.7, 0.7],
						diffuse: [0.1, 0.7, 0.7],
						specular: [0.7, 0.7, 0.7],
						illum: 10,
					}
				}
			},
			&None => mtl::WavefrontMaterial {
				name: String::from("Default material"),
				specular_exponent: 1.0,
				ambient: [0.1, 0.7, 0.7],
				diffuse: [0.1, 0.7, 0.7],
				specular: [0.7, 0.7, 0.7],
				illum: 10,
			}
		};

		(vertices, material)
	}
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
	do_parse!(num: digit >> (str::from_utf8(num).unwrap().parse::<u32>().unwrap() - 1))
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
		(Vector3::new(u, v, w.unwrap_or(0.0)))
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
		(Face {
			vertices: [ v1, v2, v3 ],
			normals: [ vn1, vn2, vn3 ],
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
		(WavefrontObject {
			name: name,
			vertices: vertices,
			normals: normals,
			texcoords: textcoords,
			material: material,
			smoothing: smoothing,
			faces: faces,
		})
	)
);

named!(obj_file<&[u8], (Vec<WavefrontObject>, Option<String>)>,
	do_parse!(
		many0!(comment) >>
		mtllib: opt!(mtllib) >>
		objects: many1!(vertex_group) >>
		((
			objects,
			mtllib
		))
	)
);

fn parse(input: &[u8]) -> Result<(Vec<WavefrontObject>, Option<String>), String> {
	match obj_file(input) {
		IResult::Done(_, object) => Ok(object),
		IResult::Incomplete(need) => {
			Err(format!("Incomplete, {:?}", need))
		},
		IResult::Error(err) => {
			Err(format!("Some error: {:?}", err))
		}
	}
}

pub fn load(pathname: &str) -> Result<WavefrontModel, String> {
	let path = Path::new(pathname);
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

	let (objects, mtllib) = parse(&s.as_bytes()).unwrap();

	let materials = match mtllib {
		Some(mtl_filename) => {
			let mtl_path = path.with_file_name(mtl_filename);
			let mtl_display = mtl_path.display();

			let mut mtl_file = match File::open(&mtl_path) {
				Err(why) => panic!("couldn't open {}: {}", mtl_display, why.description()),
				Ok(file) => file,
			};

			let mut mtl_s = String::new();

			match mtl_file.read_to_string(&mut mtl_s) {
				Err(why) => panic!("couldn't read {}: {}", mtl_display, why.description()),
				Ok(_) => ()
			}

			Some(mtl::parse(&mtl_s.as_bytes()).unwrap())
		},
		None => None
	};

	Ok(WavefrontModel {
		objects: objects,
		materials: materials,
	})
}