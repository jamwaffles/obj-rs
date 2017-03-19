use nom::{ space, digit, line_ending, IResult, not_line_ending };
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

use super::mtl;

#[derive(Debug, Clone)]
struct Face {
	vertices: [u32; 3],
	normals: [u32; 3],
}

#[derive(Debug, Clone)]
struct WavefrontObject {
	name: String,
	material_name: Option<String>,
	smoothing: Option<bool>,
	faces: Vec<Face>,
}

#[derive(Debug)]
pub struct WavefrontModelData {
	objects: Vec<WavefrontObject>,
	vertices: Vec<[f32; 3]>,
	normals: Vec<[f32; 3]>,
	texcoords: Vec<[f32; 3]>,
}

#[derive(Debug)]
pub struct WavefrontModel {
	materials: Option<mtl::WavefrontMaterials>,
	objects: Vec<WavefrontObject>,
	vertices: Vec<[f32; 3]>,
	normals: Vec<[f32; 3]>,
	texcoords: Vec<[f32; 3]>,
}

#[derive(Copy, Clone, Debug)]
pub struct BufferVertex {
    position: [ f32; 3 ],
	normal: [ f32; 3 ],
}

implement_vertex!(BufferVertex, position, normal);

impl WavefrontModel {
	pub fn to_vertices(&self) -> Vec<(Vec<BufferVertex>, mtl::WavefrontMaterial)> {
		self.objects.iter().map(|ref object| {
			let vertices = object.faces.iter().flat_map(|f| {
				let v1 = self.vertices.get(f.vertices[0] as usize).expect(&format!("Could not get v1 {}", f.vertices[0]));
				let v2 = self.vertices.get(f.vertices[1] as usize).expect(&format!("Could not get v2 {}", f.vertices[1]));
				let v3 = self.vertices.get(f.vertices[2] as usize).expect(&format!("Could not get v3 {}", f.vertices[2]));

				let vn1 = self.normals.get(f.normals[0] as usize).expect(&format!("Could not get vn1 {}", f.normals[0]));
				let vn2 = self.normals.get(f.normals[1] as usize).expect(&format!("Could not get vn2 {}", f.normals[1]));
				let vn3 = self.normals.get(f.normals[2] as usize).expect(&format!("Could not get vn3 {}", f.normals[2]));

				vec![
					BufferVertex { position: *v1, normal: *vn1 },
					BufferVertex { position: *v2, normal: *vn2 },
					BufferVertex { position: *v3, normal: *vn3 },
				]
			}).collect();

			let mat = match object.material_name {
				Some(ref mat) => (*mat).clone(),
				None => String::from("Material")
			};

			let material = match &self.materials {
				&Some(ref materials) => {
					match &materials.get(&mat) {
						&Some(ref mat) => (*mat).clone(),
						&None => mtl::WavefrontMaterial {
							name: String::from("Default material"),
							specular_exponent: 1.0,
							ambient: [1.0, 0.0, 0.0],
							diffuse: [1.0, 0.0, 0.0],
							specular: [0.7, 0.7, 0.7],
						}
					}
				},
				&None => mtl::WavefrontMaterial {
					name: String::from("Default material"),
					specular_exponent: 1.0,
					ambient: [1.0, 0.0, 0.0],
					diffuse: [1.0, 0.0, 0.0],
					specular: [0.7, 0.7, 0.7],
				}
			};

			(vertices, material)
		}).collect()
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

named!(parse_vector3<&[u8], [f32; 3]>,
	do_parse!(
		x: parse_float >>
		space >>
		y: parse_float >>
		space >>
		z: parse_float >>
		line_ending >>
		([ x, y, z ])
	)
);

named!(vertex <&[u8], [f32; 3]>, do_parse!(tag!("v") >> space >> vector: parse_vector3 >> (vector)));
named!(normal <&[u8], [f32; 3]>, do_parse!(tag!("vn") >> space >> vector: parse_vector3 >> (vector)));
named!(texcoord<&[u8], [f32; 3]>,
	do_parse!(
		tag!("vt") >>
		space >>
		u: parse_float >>
		space >>
		v: parse_float >>
		opt!(space) >>
		w: opt!(parse_float) >>
		([ u, v, w.unwrap_or(0.0) ])
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

#[derive(Debug)]
enum FileEntity {
	Vertex([f32; 3]),
	Normal([f32; 3]),
	TexCoord([f32; 3]),
	Face(Face),
	Material(String),
	Smoothing(bool),
	Object(String),
	MatLib(String),
	Ignore
}

named!(entity<&[u8], FileEntity>, alt!(
	vertex => { |v| FileEntity::Vertex(v) } |
	normal => { |n| FileEntity::Normal(n) } |
	face => { |f| FileEntity::Face(f) } |
	texcoord => { |t| FileEntity::TexCoord(t) } |
	usemtl => { |m| FileEntity::Material(m) } |
	smoothing => { |s| FileEntity::Smoothing(s) } |
	mtllib => { |m| FileEntity::MatLib(m) } |
	object_start => { |o| FileEntity::Object(String::from(o)) } |

	take_until_and_consume!("\n") => { |_| FileEntity::Ignore }
));

named!(file<&[u8], Vec<FileEntity>>, many1!(entity));

fn parse(input: &[u8]) -> Result<(WavefrontModelData, Option<String>), String> {


	match file(input) {
		IResult::Done(_, lines) => {
			let mut vertices = Vec::new();
			let mut normals = Vec::new();
			let mut texcoords = Vec::new();
			let mut objects = Vec::new();
			let mut mtl_lib: Option<String> = None;

			for line in lines.into_iter() {
				match line {
					FileEntity::Vertex(ref v) => vertices.push(*v),
					FileEntity::Normal(ref n) => normals.push(*n),
					FileEntity::TexCoord(ref t) => texcoords.push(*t),
					FileEntity::Object(ref o) => {
						objects.push(WavefrontObject {
							name: (*o).clone(),
							material_name: None,
							smoothing: None,
							faces: Vec::new(),
						})
					},
					FileEntity::MatLib(ref m_filename) => mtl_lib = Some((*m_filename).clone()),
					FileEntity::Face(ref f) => { objects.last_mut().unwrap().faces.push((*f).clone()) },
					FileEntity::Material(ref m) => { objects.last_mut().	unwrap().material_name = Some((*m).clone()) },
					FileEntity::Smoothing(ref s) => { objects.last_mut().unwrap().smoothing = Some(*s) },
					FileEntity::Ignore => (),
				}
			}

			Ok((WavefrontModelData {
				vertices: vertices,
				normals: normals,
				texcoords: texcoords,
				objects: objects,
			}, mtl_lib))
		},
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

	let (model, mtllib) = parse(&s.as_bytes()).unwrap();

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
		materials: materials,
		vertices: model.vertices,
		normals: model.normals,
		texcoords: model.texcoords,
		objects: model.objects,
	})
}