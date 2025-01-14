use color_eyre::Result;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    self,
    layout::{Direction, Rect},
    prelude::{Backend, CrosstermBackend, Terminal},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block},
};
use ringbuf::{traits::*, HeapCons, HeapRb};
use std::time::Duration;
use std::{
    io::{self, stdout},
    thread,
};

fn main() -> Result<()> {
    // TODO retrive real infos from ALSA
    let rb = HeapRb::<(f32, f32, f32, f32)>::new(10);
    let (mut prod, mut cons) = rb.split();
    std::thread::spawn(move || {
        let mut counter: f32 = 0.0;
        while counter < 50.0 {
            let l1 = 0.5 * (counter + 0.5f32 * std::f32::consts::PI).sin() + 0.5;
            let r1 = 0.5 * (counter + 1.0f32 * std::f32::consts::PI).sin() + 0.5;
            let l2 = 0.5 * (counter + 1.5f32 * std::f32::consts::PI).sin() + 0.5;
            let r2 = 0.5 * (counter + 0.5f32 * std::f32::consts::PI).sin() + 0.5;
            let _ = prod.try_push((l1, r1, l2, r2)).unwrap();
            thread::sleep(Duration::from_millis(50));
            counter += 0.03;
        }
    });

    initialize_panic_handler();
    color_eyre::install()?;
    let terminal = init_tui()?;
    let _app_result = run(terminal, &mut cons);
    restore_tui()?;
    Ok(())
}

pub fn init_tui() -> io::Result<Terminal<impl Backend>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore_tui() -> io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run(
    mut terminal: Terminal<impl Backend>,
    rb_cons: &mut HeapCons<(f32, f32, f32, f32)>,
) -> Result<()> {
    let mut titles = ["card0", "card1"];
    let number_of_devices = 2;
    let number_of_channels: usize = 10;
    let mut channel_values: Vec<f64> = vec![0.1, 0.15, 0.2, 0.225];
    loop {
        if let Some((l1, r1, l2, r2)) = rb_cons.try_pop() {
            channel_values = vec![l1 as f64, r1 as f64, l2 as f64, r2 as f64, 0.0f64];
        }
        terminal.draw(|frame| {
            let mut used_y: u16 = 0;
            for i in 0..number_of_devices {
                let area = Rect::new(
                    0,
                    used_y,
                    frame.size().width,
                    channel_values.len().try_into().unwrap(),
                );
                used_y += <usize as TryInto<u16>>::try_into(channel_values.len()).unwrap();
                let bars: Vec<Bar> = channel_values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Bar::default()
                            .value(((100.0 * *value).min(100.0) as i32).try_into().unwrap())
                            .label(Line::from(format!("{index}")))
                            .style(Style::default().fg(Color::Red))
                            .value_style(Style::default().bg(Color::Red).fg(Color::White))
                            .text_value(format!("{index}: {value}"))
                    })
                    .collect();
                let title = &titles[i];
                let barchart = BarChart::default()
                    .data(BarGroup::default().bars(&bars))
                    .block(Block::new().title(format!("{title}")))
                    .bar_width(1)
                    .bar_gap(0)
                    .label_style(Style::new().red().on_white())
                    .max(100)
                    .direction(Direction::Horizontal);
                frame.render_widget(barchart, area);
            }
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;
            match event {
                event::Event::Key(key) => {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
                _ => {
                    println!("unknown event");
                }
            }
        }
    }
    Ok(())
}

// Panic hook
pub fn initialize_panic_handler() {
    use better_panic::Settings;
    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
    }));
}
