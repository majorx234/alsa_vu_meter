use std::env::args;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::layout::{Constraint, Layout};

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("hello terminal");
    Ok(())
}

fn run() -> Result<()> {
    Ok(())
}
