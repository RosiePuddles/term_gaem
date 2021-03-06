use std::{io::{stdout, Write}};
use std::collections::HashMap;
use std::process::exit;
use std::time::Duration;

use crossterm::{
	cursor::{self, Show},
	event::{Event, KeyCode, poll, read}, ExecutableCommand, execute, QueueableCommand,
	Result as result,
	style::{Color, Print, SetBackgroundColor, SetForegroundColor},
	terminal::{self, LeaveAlternateScreen},
};
use enum_iterator::IntoEnumIterator;
use regex::Regex;

use crate::{Ingredient, Node};
use crate::resipee::Resipee;

const SPACE: char = 'ᄢ';

pub struct Grid {
	grid: Vec<Vec<Option<Node>>>,
	width: usize,
	height: usize,
	console: [Option<String>; 3],
	console_is_err: [bool; 3],
	resipees: HashMap<u32, Resipee>,
}

impl Grid {
	pub fn new(width: usize, height: usize, resipees: HashMap<u32, Resipee>) -> Grid {
		Grid { grid: vec![vec![None; width + 3]; height + 4], width: width - 1, height: height - 1, console: [None, None, None], console_is_err: [false, false, false], resipees }
	}
	
	pub fn set_node(&mut self, x: usize, y: usize, node: Node) -> Result<(), ()> {
		if y >= self.height || x >= self.grid.first().unwrap().len() {
			return Err(());
		}
		self.grid[y][x] = Some(node);
		self.update(x, y);
		Ok(())
	}
	
	fn update(&mut self, x: usize, y: usize) {
		let node_at_pos = match self.get_node_at_pos(x, y) {
			Some(n) => *n,
			None => return
		};
		match node_at_pos {
			// Terminal nodes
			Node::Out(_) => {
				self.grid[y][x] = Some(if x != 0 {
					match self.get_node_at_pos(x - 1, y) {
						Some(n) => Node::Out(n.get_ingredient(self.resipees.clone())),
						None => Node::Out(Ingredient::None)
					}
				} else {
					Node::Out(Ingredient::None)
				})
			}
			Node::PowerRight => {
				if x != 0 {
					self.grid[y][x] = match self.get_node_at_pos(x - 1, y) {
						Some(n) => match *n {
							Node::Split(i, _) => Some(Node::Split(i, true)),
							_ => Some(*n)
						}
						None => Some(node_at_pos)
					}
				}
			}
			Node::PowerLeft => {
				if x != self.width {
					self.grid[y][x] = match self.get_node_at_pos(x + 1, y) {
						Some(n) => match *n {
							Node::Merge(i, _) => Some(Node::Merge(i, true)),
							_ => Some(*n)
						}
						None => Some(node_at_pos)
					}
				}
			}
			// Non-terminal nodes
			Node::Pipe(_, t) => {
				if t == 0 || t == 1 || t == 2 {
					if x != 0 {
						self.grid[y][x] = Some(Node::Pipe(match self.get_node_at_pos(x - 1, y) {
							Some(n) => match *n {
								Node::Merge(_, _) | Node::Comb1(_, _, _, _) | Node::Comb2(_, _, _, _) | Node::In(_) => n.get_ingredient(self.resipees.clone()),
								Node::Pipe(i, t) => {
									if t == 0 || t == 3 || t == 4 {
										i
									} else {
										Ingredient::None
									}
								}
								_ => Ingredient::None
							}
							None => Ingredient::None
						}, t))
					}
				} else if t == 3 {
					if y != self.height {
						self.grid[y][x] = Some(Node::Pipe(match self.get_node_at_pos(x, y + 1) {
							Some(n) => {
								match *n {
									Node::Split(i, _) => i,
									_ => Ingredient::None
								}
							}
							None => Ingredient::None
						}, 3))
					}
				} else {
					if y != 0 {
						self.grid[y][x] = Some(Node::Pipe(match self.get_node_at_pos(x, y - 1) {
							Some(n) => {
								match *n {
									Node::Split(i, _) => i,
									_ => Ingredient::None
								}
							}
							None => Ingredient::None
						}, 4))
					}
				}
				if t == 0 || t == 3 || t == 4 {
					if x != self.width {
						self.update(x + 1, y);
					}
				} else if t == 1 {
					if y != 0 {
						self.update(x, y - 1);
					}
				} else if y != self.height {
					self.update(x, y + 1);
				}
			}
			Node::Comb1(_, _, _, l) | Node::Comb2(_, _, _, l) => {
				let node_type = match node_at_pos {
					Node::Comb1(_, _, _, _) => Node::Comb1,
					_ => Node::Comb2
				};
				let mut top = Ingredient::None;
				let mut left = Ingredient::None;
				let mut bottom = Ingredient::None;
				if y != 0 {
					top = match self.get_node_at_pos(x, y - 1) {
						Some(n) => match *n {
							Node::Split(i, _) => i,
							Node::Pipe(i, t) => {
								if t == 2 { i } else { Ingredient::None }
							}
							_ => Ingredient::None
						}
						None => Ingredient::None
					}
				}
				if x != 0 {
					left = match self.get_node_at_pos(x - 1, y) {
						Some(n) => match *n {
							Node::In(_) | Node::Comb1(_, _, _, _) | Node::Comb2(_, _, _, _) | Node::Merge(_, _) => n.get_ingredient(self.resipees.clone()),
							Node::Pipe(i, t) => {
								if t == 0 || t == 3 || t == 4 {
									i
								} else {
									Ingredient::None
								}
							}
							_ => Ingredient::None
						}
						None => Ingredient::None
					}
				}
				if y != self.height {
					bottom = match self.get_node_at_pos(x, y + 1) {
						Some(n) => match *n {
							Node::Split(i, _) => i,
							Node::Pipe(i, t) => {
								if t == 1 { i } else { Ingredient::None }
							}
							_ => Ingredient::None
						}
						None => Ingredient::None
					}
				}
				self.grid[y][x] = Some(node_type(top, left, bottom, l));
				if x != self.width {
					self.update(x + 1, y);
				}
			}
			Node::Split(_, o) => {
				let mut oc = o;
				let mut ing = Ingredient::None;
				if x != 0 {
					match self.get_node_at_pos(x - 1, y) {
						Some(n) => ing = match *n {
							Node::In(_) | Node::Merge(_, _) | Node::Comb1(_, _, _, _) | Node::Comb2(_, _, _, _) => n.get_ingredient(self.resipees.clone()),
							Node::Pipe(i, t) => {
								if t == 0 || t == 3 || t == 4 {
									i
								} else {
									Ingredient::None
								}
							}
							_ => Ingredient::None
						},
						None => {}
					}
				}
				if x != self.width {
					oc = match self.get_node_at_pos(x + 1, y) {
						Some(n) => match *n {
							Node::PowerRight => true,
							_ => false
						}
						None => false
					}
				}
				self.grid[y][x] = Some(Node::Split(ing, oc));
				if y != 0 {
					self.update(x, y - 1);
				}
				if y != self.height {
					self.update(x, y + 1);
				}
			}
			Node::Merge(_, o) => {
				let mut oc = o;
				let mut ing: Option<Ingredient> = None;
				if y != 0 {
					match self.get_node_at_pos(x, y - 1) {
						Some(n) => match *n {
							Node::Pipe(i, t) => {
								if t == 2 {
									ing = Some(i)
								}
							}
							_ => {}
						}
						None => {}
					}
				}
				if ing.is_none() && y != self.height {
					match self.get_node_at_pos(x, y + 1) {
						Some(n) => match *n {
							Node::Pipe(i, t) => {
								if t == 1 {
									ing = Some(i)
								}
							}
							_ => {}
						}
						None => {}
					}
				}
				self.grid[y][x] = Some(Node::Merge(if ing.is_some() { ing.unwrap() } else { Ingredient::None }, oc));
				if x != self.width {
					self.update(x + 1, y);
				}
			}
			Node::In(_) => {
				if x != self.width {
					self.update(x + 1, y);
				}
			}
		}
	}
	
	fn get_node_at_pos(&self, x: usize, y: usize) -> &Option<Node> {
		self.grid.get(y).unwrap().get(x).unwrap()
	}
	
	pub fn print_to_stdout(&self, current_command: String) -> result<()> {
		let mut stdout = stdout();
		let mut colours = Ingredient::into_enum_iter();
		print!("{}", colours.len());
		colours.next();
		stdout.execute(terminal::Clear(terminal::ClearType::All))?
			.queue(cursor::MoveTo(0, 0))?;
		for y in 0..self.height {
			for x in 0..self.width {
				let node = self.get_node_at_pos(x, y);
				stdout
					.queue(SetForegroundColor(match node {
						Some(n) => n.col(self.resipees.clone()),
						None => Color::Reset
					})
					)?
					.queue(Print(format!("{}", match node {
						Some(n) => n.char(),
						None => SPACE
					})))?;
			}
			match colours.next() {
				Some(ing) => stdout.queue(cursor::MoveRight(1))?
					.queue(Print(format!("{:>2}", y + 1)))?
					.queue(SetForegroundColor(ing.get_colour()))?
					.queue(Print(ing.char()))?
					.queue(SetForegroundColor(Color::Reset))?
					.queue(Print(format!("{:?}", ing)))?,
				None => &mut stdout
			};
			stdout.queue(cursor::MoveToNextLine(1))?;
		}
		
		stdout.queue(cursor::MoveTo(0, (self.height + 1) as u16))?
			.queue(SetForegroundColor(Color::Reset))?
			.queue(Print(current_command));
		for console_line in 0..3 {
			stdout.queue(cursor::MoveToNextLine(1))?
				.queue(SetForegroundColor(if *self.console_is_err.get(console_line).unwrap() {
					Color::Red
				} else {
					Color::Reset
				}))?
				.queue(Print(match self.console.get(console_line).unwrap() {
					Some(s) => s.clone(),
					None => String::new()
				}))?
				.queue(SetForegroundColor(Color::Reset));
		}
		
		stdout.flush()?;
		
		Ok(())
	}
	
	pub fn execute_command(&mut self, current_command: String) {
		let mut chars = current_command.chars();
		match chars.nth(0) {
			Some(c) => {
				if c != ':' { return; }
			}
			None => return
		}
		// Have to get the 0th character because the previous char ':' was removed, shifting the op_code to index 0
		let op_code = match chars.nth(0) {
			Some(c) => c,
			None => return
		};
		match op_code {
			'q' => {
				execute!(stdout(), LeaveAlternateScreen, Show);
				exit(0)
			}
			'p' => {
				let cap = match Regex::new(r";(\w+)(\((\w{1,2})\))?").unwrap().captures(&*current_command) {
					Some(c) => c,
					None => {
						self.new_error(format!("Unknown command '{}'", current_command));
						return;
					}
				};
				let node = match &cap[1] {
					"i" => {
						if cap.get(3).is_none() {
							self.new_error("Expected 3 arguments for ':p;i' command".to_string());
							return
						}
						Node::In(match &cap[3].parse::<u16>() {
							Ok(v) => Ingredient::u16_to_ing(*v),
							Err(_) => {
								self.new_error(format!("Cannot parse '{}' as u16", &cap[3]));
								return;
							}
						})
					}
					"o" => Node::Out(Ingredient::None),
					"P" => {
						if cap.get(3).is_none() {
							self.new_error("Expected 3 arguments for ':p;P' command".to_string());
							return
						}
						match &cap[3] {
							"l" => Node::PowerLeft,
							"r" => Node::PowerRight,
							_ => {
								self.new_error(format!("Expected either 'l' or 'r'; found '{}'", &cap[3]));
								return;
							}
						}
					}
					"c1" => {
						if cap.get(3).is_none() {
							self.new_error("Expected 3 arguments for ':p;c1' command".to_string());
							return
						}
						match &cap[3].parse::<u8>() {
							Ok(v) => if v > &0 && v < &3 {
								Node::Comb1(Ingredient::None, Ingredient::None, Ingredient::None, *v - 1)
							} else {
								self.new_error(format!("Comb1 has a max level of level 2. Given level {}", v));
								return;
							},
							Err(_) => {
								self.new_error(format!("Cannot parse '{}' as u16", &cap[3]));
								return;
							}
						}
					}
					"c2" => {
						if cap.get(3).is_none() {
							self.new_error("Expected 3 arguments for ':p;c2' command".to_string());
							return
						}
						match &cap[3].parse::<u8>() {
							Ok(v) => if v > &0 && v < &3 {
								Node::Comb2(Ingredient::None, Ingredient::None, Ingredient::None, *v - 1)
							} else {
								self.new_error(format!("Comb2 has a max level of level 2. Given level {}", v));
								return;
							},
							Err(_) => {
								self.new_error(format!("Cannot parse '{}' as u16", &cap[3]));
								return;
							}
						}
					}
					"s" => Node::Split(Ingredient::None, false),
					"m" => Node::Merge(Ingredient::None, false),
					"p" => {
						if cap.get(3).is_none() {
							self.new_error("Expected 3 arguments for ':p;p' command".to_string());
							return
						}
						Node::Pipe(Ingredient::None, match &cap[3] {
						"lr" => 0,
						"lu" => 1,
						"ld" => 2,
						"dr" => 3,
						"ur" => 4,
						_ => {
							self.new_error(format!("Unknown pipe code '{}'", &cap[3]));
							return;
						}
					})}
					_ => {
						self.new_error(format!("Unknown node command '{}'", current_command));
						return;
					}
				};
				self.place(node);
			}
			'd' => { self.delete(); }
			'i' => { self.info(); }
			'r' => {
				match chars.nth(0) {
					Some(c) => if c != ';' {
						self.new_error(format!("Expected ';', found {}", c));
						return;
					}
					None => {
						self.new_error("Expected a ';' after ':r'".to_string());
						return;
					}
				}
				match chars.as_str().parse::<u16>() {
					Ok(v) => {
						let out = Ingredient::u16_to_ing(v);
						let (ing_vec, mac_req) = out.resipee();
						let ing_vec = ing_vec.iter().map(|arg| format!("{:?}", arg)).collect::<Vec<String>>();
						self.new_info(format!("{:?} -> Ingredients: {} | {}", out, ing_vec.join(", "), match mac_req {
							Some(mr) => format!("{:?}", mr),
							None => "Base ingredient. Comes form input only".to_string()
						}))
					}
					Err(_) => {
						self.new_error(format!("Cannot parse '{}' as u16", chars.as_str()));
						return;
					}
				}
			}
			_ => self.new_error(format!("Unknown command '{}'", current_command))
		}
	}
	
	fn new_error(&mut self, err: String) {
		self.console = [Some(err), self.console.get(0).unwrap().clone(), self.console.get(1).unwrap().clone()];
		self.console_is_err = [true, self.console_is_err.get(0).unwrap().clone(), self.console_is_err.get(1).unwrap().clone()];
	}
	fn new_info(&mut self, info: String) {
		self.console = [Some(info), self.console.get(0).unwrap().clone(), self.console.get(1).unwrap().clone()];
		self.console_is_err = [false, self.console_is_err.get(0).unwrap().clone(), self.console_is_err.get(1).unwrap().clone()];
	}
	
	fn place(&mut self, node: Node) -> result<()> {
		execute!(stdout(), Show)?;
		let mut x = 0u16;
		let mut y = 0u16;
		loop {
			if poll(Duration::from_millis(500))? {
				match read()? {
					Event::Key(key) => {
						match key.code {
							KeyCode::Left => if x != 0 {
								x -= 1
							}
							KeyCode::Right => if x != (self.width - 1) as u16 {
								x += 1
							}
							KeyCode::Up => if y != 0 {
								y -= 1
							}
							KeyCode::Down => if y != self.height as u16 {
								y += 1
							}
							KeyCode::Enter => {
								self.set_node(x as usize, y as usize, node);
								self.update(x as usize, y as usize);
								execute!(stdout(), cursor::Hide)?;
								break;
							}
							KeyCode::Esc => {
								execute!(stdout(), cursor::Hide)?;
								break;
							}
							_ => {}
						}
					}
					_ => {}
				}
			}
			self.print_to_stdout(String::new())?;
			let mut stdout = stdout();
			stdout.queue(cursor::MoveTo(2 * x, y))?;
			stdout.flush()?
		}
		Ok(())
	}
	
	fn delete(&mut self) -> result<()> {
		let mut stdout = stdout();
		execute!(stdout, Show)?;
		let mut x = 0u16;
		let mut y = 0u16;
		loop {
			if poll(Duration::from_millis(500))? {
				match read()? {
					Event::Key(key) => {
						match key.code {
							KeyCode::Left => if x != 0 {
								x -= 1
							}
							KeyCode::Right => if x != (self.width - 1) as u16 {
								x += 1
							}
							KeyCode::Up => if y != 0 {
								y -= 1
							}
							KeyCode::Down => if y != self.height as u16 {
								y += 1
							}
							KeyCode::Enter => {
								let previous = self.get_node_at_pos(x as usize, y as usize).clone();
								self.grid[y as usize][x as usize] = None;
								match previous {
									Some(n) => match n {
										Node::In(_) | Node::PowerRight | Node::Merge(_, _) | Node::Comb1(_, _, _, _) | Node::Comb2(_, _, _, _) => {
											if x != (self.width - 1) as u16 {
												self.update((x + 1) as usize, y as usize);
											}
										}
										Node::PowerLeft | Node::Split(_, _) => {
											if x != 0 {
												self.update((x - 1) as usize, y as usize);
											}
										}
										Node::Pipe(_, t) => match t {
											0 | 3 | 4 => {
												if x != self.width as u16 {
													self.update((x + 1) as usize, y as usize);
												}
											}
											1 => {
												if y != 0 {
													self.update(x as usize, (y - 1) as usize);
												}
											}
											_ => {
												if y != self.height as u16 {
													self.update(x as usize, (y + 1) as usize);
												}
											}
										}
										_ => {}
									}
									None => {}
								}
								self.update(x as usize, y as usize);
								execute!(stdout, cursor::Hide)?;
								break;
							}
							KeyCode::Esc => {
								execute!(stdout, cursor::Hide)?;
								break;
							}
							_ => {}
						}
					}
					_ => {}
				}
			}
			stdout.queue(cursor::MoveTo(2 * x, y))?;
			stdout.flush()?
		}
		Ok(())
	}
	
	fn info(&mut self) -> result<()> {
		let mut stdout = stdout();
		execute!(stdout, Show)?;
		let mut x = 0u16;
		let mut y = 0u16;
		loop {
			if poll(Duration::from_millis(500))? {
				match read()? {
					Event::Key(key) => {
						match key.code {
							KeyCode::Left => if x != 0 {
								x -= 1
							}
							KeyCode::Right => if x != (self.width - 1) as u16 {
								x += 1
							}
							KeyCode::Up => if y != 0 {
								y -= 1
							}
							KeyCode::Down => if y != self.height as u16 {
								y += 1
							}
							KeyCode::Enter => {
								self.new_info(match self.get_node_at_pos(x as usize, y as usize) {
									Some(n) => n.info(self.resipees.clone()),
									None => String::from("No node")
								});
								execute!(stdout, cursor::Hide)?;
								break;
							}
							KeyCode::Esc => {
								execute!(stdout, cursor::Hide)?;
								break;
							}
							_ => {}
						}
					}
					_ => {}
				}
			}
			stdout.queue(cursor::MoveTo(2 * x, y))?;
			stdout.flush()?
		}
		Ok(())
	}
}
