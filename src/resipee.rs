use std::collections::HashMap;

use crossterm::style::Color;
use enum_iterator::IntoEnumIterator;
use crate::Ingredient::*;

use crate::Node;
use crate::Node::{Comb1, Comb2};

#[derive(Copy, Clone)]
pub struct Resipee {
	input: [Ingredient; 3],
	pub machine: MachineRequirement,
	pub output: Ingredient,
}

#[derive(Copy, Clone)]
pub struct MachineRequirement {
	machine: Node,
	pub min_level: u8,
}

pub fn resipee_hash(machine: &Node, ings: &[Ingredient; 3]) -> u32 {
	// 2 | 3 | 5 | 7 | 11
	let mut out = match machine {
		Comb1(_, _, _, _) => 2u32,
		Comb2(_, _, _, _) => 3u32,
		_ => unreachable!()
	};
	for ing in ings {
		out *= ing.index()
	}
	out
}

pub fn generate_resipees() -> HashMap<u32, Resipee> {
	let mut resipees: HashMap<u32, Resipee> = HashMap::new();
	let r1 = Resipee { input: [Hot, Water, Milk], machine: MachineRequirement { machine: Comb1(None, None, None, 0), min_level: 0}, output: Coffee };
	let r2 = Resipee { input: [Coffee, Cat, Pink], machine: MachineRequirement { machine: Comb2(None, None, None, 0), min_level: 0}, output: Metal };
	let r3 = Resipee { input: [Metal, Hot, None], machine: MachineRequirement { machine: Comb1(None, None, None, 0), min_level: 1}, output: OtherMetal };
	let r4 = Resipee { input: [Pink, Cold, Milk], machine: MachineRequirement { machine: Comb1(None, None, None, 0), min_level: 1}, output: Vodka };
	for r in [r1, r2, r3, r4] {
		resipees.insert(resipee_hash(&r.machine.machine, &r.input), r);
	}
	resipees
}

#[derive(Copy, Clone, IntoEnumIterator, Debug)]
pub enum Ingredient {
	None,
	Hot,
	Cold,
	Metal,
	Milk,
	OtherMetal,
	Water,
	Pink,
	Coffee,
	Vodka,
	Cat,
}

impl Ingredient {
	pub fn get_colour(&self) -> Color {
		match self {
			None => Color::Reset,
			Hot => Color::Rgb { r: 255, g: 166, b: 0 },
			Cold => Color::Rgb{ r: 34, g: 165, b: 213 },
			Metal => Color::Rgb { r: 255, g: 166, b: 77 },
			Milk => Color::Rgb { r: 255, g: 255, b: 255 },
			OtherMetal => Color::Rgb { r: 255, g: 0, b: 0 },
			Water => Color::Rgb { r: 36, g: 67, b: 245 },
			Pink => Color::Rgb { r: 255, g: 0, b: 145 },
			Coffee => Color::Rgb { r: 123, g: 100, b: 26 },
			Vodka => Color::Rgb { r: 208, g: 196, b: 25 },
			Cat => Color::Rgb { r: 136, g: 136, b: 136 }
		}
	}
	
	pub fn index(&self) -> u32 {
		match self {
			None => 1,
			Hot => 13,
			Cold => 17,
			Metal => 19,
			Milk => 23,
			OtherMetal => 29,
			Water => 31,
			Pink => 37,
			Coffee => 41,
			Vodka => 43,
			Cat => 47
		}
	}
	
	pub fn char(&self) -> char {
		match self {
			None => ' ',
			Hot => 'ᄥ',
			Cold => 'ᄣ',
			Metal => 'ᄪ',
			Milk => 'ᄧ',
			OtherMetal => 'ᄫ',
			Water => 'ᄨ',
			Pink => 'ᄤ',
			Coffee => 'ᄩ',
			Vodka => 'ᄬ',
			Cat => 'ᄦ'
		}
	}
	
	pub fn u16_to_ing(v: u16) -> Ingredient {
		match v {
			0 => None,
			1 => Hot,
			2 => Cold,
			3 => Metal,
			4 => Milk,
			5 => OtherMetal,
			6 => Water,
			7 => Pink,
			8 => Coffee,
			9 => Vodka,
			10 => Cat,
			_ => unreachable!()
		}
	}
}
