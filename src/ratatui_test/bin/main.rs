use std::{env::args, io::stdout};

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    self,
    layout::{Constraint, Layout, Rect},
    prelude::{CrosstermBackend, Stylize, Terminal, TerminalOptions},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block},
    Viewport,
};

fn main() -> Result<()> {
    // TODO retrive real infos from ALSA
    let titles = ["card0", "card1"];
    let number_of_devices = 2;
    let number_of_channels: usize = 10;
    let channel_values: Vec<f64> = vec![
        0.1, 0.15, 0.2, 0.225, 0.25, 0.275, 0.3, 0.3125, 0.325, 0.3375,
    ];
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    color_eyre::install()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.draw(|frame| {
        for i in 0..number_of_devices {
            let area = Rect::new(0, i as u16, frame.size().width, 1);
            let bars: Vec<Bar> = channel_values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    Bar::default()
                        .value(((100.0 * *value) as i32).try_into().unwrap())
                        .label(Line::from(format!("{index}")))
                })
                .collect();
            let title = titles[i];
            let barchart = BarChart::default()
                .data(BarGroup::default().bars(&bars))
                .block(Block::new().title(format!("{title}")))
                .bar_width(5);
            frame.render_widget(barchart, area);
        }
    });
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run() -> Result<()> {
    Ok(())
}
