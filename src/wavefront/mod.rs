mod obj;
mod mtl;

use self::obj::{ parse, WavefrontModel };

pub fn parse_obj_string(input: &String) -> Result<WavefrontModel, String> {
	parse(input.as_bytes())
}