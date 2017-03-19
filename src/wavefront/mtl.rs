use nom::{ space, digit, line_ending, IResult, not_line_ending };
use nom;
use std::str;
use std::collections::HashMap;

pub type WavefrontMaterials = HashMap<String, WavefrontMaterial>;

#[derive(Debug, Clone)]
pub struct WavefrontMaterial {
	pub name: String,
	pub specular_exponent: f32,
	pub ambient: [f32; 3],
	pub diffuse: [f32; 3],
	pub specular: [f32; 3],
}

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

named!(material_start<&[u8], String>,
	do_parse!(
		tag!("newmtl") >>
		space >>
		name: not_line_ending >>
		line_ending >>
		(String::from(str::from_utf8(name).unwrap_or("")))
	)
);

// named!(material<&[u8], WavefrontMaterial>,
// 	do_parse!(
// 		name: material_start >>
// 		tag!("Ns") >> space >> specular_exponent: parse_float >> line_ending >>
// 		tag!("Ka") >> space >> ambient: parse_vector3 >>
// 		tag!("Kd") >> space >> diffuse: parse_vector3 >>
// 		tag!("Ks") >> space >> specular: parse_vector3 >>

// 		tag!("Ke") >> not_line_ending >> line_ending >>
// 		tag!("Ni") >> not_line_ending >> line_ending >>
// 		tag!("d") >> not_line_ending >> line_ending >>

// 		tag!("illum") >> space >> illum: parse_u32 >>
// 		(WavefrontMaterial {
// 			name: name,
// 			specular_exponent: specular_exponent,
// 			ambient: ambient,
// 			diffuse: diffuse,
// 			specular: specular,
// 			illum: illum,
// 		})
// 	)
// );

// named!(mtl_file<&[u8], Vec<WavefrontMaterial>>,
// 	do_parse!(
// 		many0!(comment) >>
// 		opt!(multispace) >>
// 		materials: many1!(material) >>
// 		(materials)
// 	)
// );

#[derive(Debug)]
enum FileEntity {
	Name(String),
	Ambient([f32; 3]),
	Diffuse([f32; 3]),
	Specular([f32; 3]),
	Exponent(f32),

	Ignore
}

named!(entity<&[u8], FileEntity>, alt!(
	material_start => { |name| FileEntity::Name(name) } |
	preceded!(tag!("Ns "), parse_float) => { |exp| FileEntity::Exponent(exp) } |
	preceded!(tag!("Ka "), parse_vector3) => { |a| FileEntity::Ambient(a) } |
	preceded!(tag!("Kd "), parse_vector3) => { |d| FileEntity::Diffuse(d) } |
	preceded!(tag!("Ks "), parse_vector3) => { |s| FileEntity::Specular(s) } |

	take_until_and_consume!("\n") => { |_| FileEntity::Ignore }
));

named!(file<&[u8], Vec<FileEntity>>, many1!(entity));

pub fn parse(input: &[u8]) -> Result<WavefrontMaterials, String> {
	match file(input) {
		IResult::Done(_, lines) => {
			let mut materials = Vec::new();

			let mut map = HashMap::new();

			for line in lines.into_iter() {
				match line {
					FileEntity::Name(ref name) => {
						materials.push(WavefrontMaterial {
							name: (*name).clone(),
							ambient: [ 0.0, 0.0, 0.0 ],
							diffuse: [ 0.0, 0.0, 0.0 ],
							specular: [ 0.0, 0.0, 0.0 ],
							specular_exponent: 0.0,
						})
					},
					FileEntity::Ambient(ref a) => { materials.last_mut().unwrap().ambient = *a },
					FileEntity::Diffuse(ref d) => { materials.last_mut().unwrap().diffuse = *d },
					FileEntity::Specular(ref s) => { materials.last_mut().unwrap().specular = *s },
					FileEntity::Exponent(ref exp) => { materials.last_mut().unwrap().specular_exponent = *exp },
					FileEntity::Ignore => {}
				}
			}

			for material in materials.iter() {
				map.insert(material.name.clone(), material.clone());
			}

			Ok(map)
		},
		IResult::Incomplete(need) => {
			Err(format!("Incomplete, {:?}", need))
		},
		IResult::Error(err) => {
			match err {
				nom::Err::NodePosition(kind, position, _) => Err(format!("NodePosition {:?} {:?}", kind, str::from_utf8(position).unwrap())),
				nom::Err::Position(kind, position) => Err(format!("Position {:?} {:?}", kind, str::from_utf8(position).unwrap())),
				_ => Err(format!("Some error: {:?}", err))
			}
		}
	}
}