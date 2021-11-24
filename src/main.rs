mod nodes;
mod grid;
mod resipee;

use std::io::stdout;
use std::time::Duration;
use crossterm::{execute, Result, terminal::{SetSize, size}, cursor::{Hide, DisableBlinking}};
use crossterm::event::{Event, read, poll, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen};
use crate::nodes::Node;
use crate::resipee::Ingredient;

fn main() -> Result<()> {
	execute!(stdout(), EnterAlternateScreen, Hide, DisableBlinking)?;
	let (cols, rows) = size()?;
    // Resize terminal and scroll up.
	enable_raw_mode()?;
	let mut grid = grid::Grid::new(15, 15, resipee::generate_resipees());
	let mut command = String::new();
	
	loop {
		if poll(Duration::from_millis(500))? {
			match read()? {
				Event::Key(e) => {
					match e.code {
						KeyCode::Char(c) => command.push(c),
						KeyCode::Backspace => match command.pop() {
							_ => {}
						},
						KeyCode::Enter => {
							grid.execute_command(command.clone());
							command.clear()
						},
						_ => {}
					}
				},
				_ => ()
			}
		}
		disable_raw_mode()?;
		grid.print_to_stdout(command.clone())?;
		enable_raw_mode()?;
		execute!(stdout(), SetSize(cols, rows))?;
	}
}
