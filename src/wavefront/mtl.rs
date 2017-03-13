use nom::{ space, digit, line_ending, IResult, not_line_ending, multispace };
use nom;
use nalgebra::{ Vector3 };
use std::str;
use std::collections::HashMap;

#[derive(Debug)]
pub struct WavefrontMaterial {
	name: String,
	specular_exponent: f32,
	ambient: Vector3<f32>,
	diffuse: Vector3<f32>,
	specular: Vector3<f32>,
	illum: u32,
}

named!(comment, preceded!(tag!("#"), take_until_and_consume!("\n")));

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

named!(parse_u32<u32>,
	do_parse!(num: digit >> (str::from_utf8(num).unwrap().parse::<u32>().unwrap()))
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

named!(material<&[u8], WavefrontMaterial>,
	do_parse!(
		name: material_start >>
		tag!("Ns") >> space >> specular_exponent: parse_float >> line_ending >>
		tag!("Ka") >> space >> ambient: parse_vector3 >>
		tag!("Kd") >> space >> diffuse: parse_vector3 >>
		tag!("Ks") >> space >> specular: parse_vector3 >>

		tag!("Ke") >> not_line_ending >> line_ending >>
		tag!("Ni") >> not_line_ending >> line_ending >>
		tag!("d") >> not_line_ending >> line_ending >>

		tag!("illum") >> space >> illum: parse_u32 >>
		(WavefrontMaterial {
			name: name,
			specular_exponent: specular_exponent,
			ambient: ambient,
			diffuse: diffuse,
			specular: specular,
			illum: illum,
		})
	)
);

named!(mtl_file<&[u8], Vec<WavefrontMaterial>>,
	do_parse!(
		many0!(comment) >>
		opt!(multispace) >>
		materials: many1!(material) >>
		(materials)
	)
);

pub fn parse(input: &[u8]) -> Result<HashMap<String, WavefrontMaterial>, String> {
	match mtl_file(input) {
		IResult::Done(_, materials) => {
			let mut map = HashMap::new();

			for material in materials.into_iter() {
				map.insert(material.name.clone(), material);
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