mod obj;
mod mtl;

use std::collections::HashMap;
use self::obj::{ parse, WavefrontModel };

pub fn parse_obj_string(input: &String) -> Result<WavefrontModel, String> {
	parse(input.as_bytes())
}

pub fn parse_mtl_string(input: &String) -> Result<HashMap<String, mtl::WavefrontMaterial>, String> {
	mtl::parse(input.as_bytes())
}