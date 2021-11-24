use std::collections::HashMap;
use std::fmt::Formatter;

use crossterm::style::Color;

use crate::resipee::{Ingredient, Resipee, resipee_hash};

#[derive(Copy, Clone)]
pub enum Node {
	In(Ingredient),
	Out(Ingredient),
	PowerRight,
	PowerLeft,
	Comb1(Ingredient, Ingredient, Ingredient, u8),
	Comb2(Ingredient, Ingredient, Ingredient, u8),
	Split(Ingredient, bool),
	Merge(Ingredient, bool),
	Pipe(Ingredient, u8),
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

impl Node {
	pub fn char(&self) -> char {
		match self {
			Node::In(_) => 'ᄓ',
			Node::Out(_) => 'ᄔ',
			Node::PowerRight => 'ᄕ',
			Node::PowerLeft => 'ᄖ',
			Node::Comb1(_, _, _, level) => match level {
				0 => 'ᄗ',
				1 => 'ᄘ',
				_ => unreachable!()
			}
			Node::Comb2(_, _, _, level) => match level {
				0 => 'ᄙ',
				1 => 'ᄚ',
				_ => unreachable!()
			}
			Node::Split(_, _) => 'ᄛ',
			Node::Merge(_, _) => 'ᄜ',
			Node::Pipe(_, style) => match *style {
				0 => 'ᄝ',
				1 => 'ᄞ',
				2 => 'ᄟ',
				3 => 'ᄠ',
				4 => 'ᄡ',
				_ => unreachable!()
			}
		}
	}
	
	pub fn info(&self, resipees: HashMap<u32, Resipee>) -> String {
		match self {
			Node::In(i) => format!("Input node. Input: {:?}", i),
			Node::Out(i) => format!("Output node. Output: {:?}", i),
			Node::PowerRight => String::from("Power right node"),
			Node::PowerLeft => String::from("Power left node"),
			Node::Comb1(i1, i2, i3, level) | Node::Comb2(i1, i2, i3, level) =>
				format!("Comb{} node. In top: {:?}, in left: {:?}, in bottom: {:?}, level: {}, output: {:?}",
						match self {
							Node::Comb1(_, _, _, _) => 1,
							_ => 2
						}, i1, i2, i3, level, match resipees.get(&resipee_hash(self, &[*i1, *i2, *i3])) {
						Some(r) => {
							if *level >= r.machine.min_level {
								r.output
							} else {
								Ingredient::None
							}
						}
						None => Ingredient::None
					}),
			Node::Split(i, p) => format!("Split node ({}powered). Input: {:?}", if *p { "un" } else { "" }, i),
			Node::Merge(i, p) => format!("Merge node ({}powered). Output: {:?}", if *p { "un" } else { "" }, i),
			Node::Pipe(i, style) => format!("Pipe node. Carrying: {:?}, type: {}({})", i, match *style {
				0 => "lr",
				1 => "lu",
				2 => "ld",
				3 => "dr",
				4 => "ur",
				_ => unreachable!()
			}, style)
		}
	}
	
	pub fn get_ingredient(&self, resipees: HashMap<u32, Resipee>) -> Ingredient {
		match self {
			Node::In(i) => *i,
			Node::Out(i) => *i,
			Node::PowerRight => Ingredient::None,
			Node::PowerLeft => Ingredient::None,
			Node::Comb1(i0, i1, i2, l) | Node::Comb2(i0, i1, i2, l) => {
				match resipees.get(&resipee_hash(self, &[*i0, *i1, *i2])) {
					Some(r) => {
						if *l >= r.machine.min_level {
							r.output
						} else {
							Ingredient::None
						}
					}
					None => Ingredient::None
				}
			}
			Node::Split(i, _) => *i,
			Node::Merge(i, _) => *i,
			Node::Pipe(i, _) => *i
		}
	}
	
	pub fn col(&self, resipees: HashMap<u32, Resipee>) -> Color {
		self.get_ingredient(resipees).get_colour()
	}
}

impl std::fmt::Display for Node {
	fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(_f, "{}", self.char())
	}
}
