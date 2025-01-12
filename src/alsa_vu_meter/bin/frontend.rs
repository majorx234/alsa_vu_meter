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
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block},
};
use ringbuf::{traits::*, HeapCons, HeapProd};
use std::io::{self, stdout};

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

pub type ConsumerRbf32 = HeapCons<f32>;
pub type ProducerRbf32 = HeapProd<f32>;

pub fn create_gui_thread(
    ringbuffer_left_in: ConsumerRbf32,
    ringbuffer_right_in: ConsumerRbf32,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        initialize_panic_handler();
        color_eyre::install().unwrap();
        let terminal = init_tui().unwrap();
        let _app_result = run(terminal, ringbuffer_left_in, ringbuffer_right_in).unwrap();
        restore_tui().unwrap();
    })
}

pub fn run(
    mut terminal: Terminal<impl Backend>,
    mut ringbuffer_left_in: ConsumerRbf32,
    mut ringbuffer_right_in: ConsumerRbf32,
) -> Result<()> {
    let titles = ["card0", "card1"];
    let number_of_devices = 2;
    let number_of_channels: usize = 2;
    let mut channel_values_last = vec![0.0f32, 0.0f32];
    loop {
        let left_value = if let Some(left_value) = ringbuffer_left_in.try_pop() {
            left_value
        } else {
            channel_values_last[0]
        };
        let right_value = if let Some(right_value) = ringbuffer_right_in.try_pop() {
            right_value
        } else {
            channel_values_last[1]
        };
        let channel_values: Vec<f32> = vec![left_value, right_value];
        channel_values_last = channel_values.clone();
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
                            .value(((100.0 * *value) as i32).try_into().unwrap())
                            .label(Line::from(format!("{index}")))
                    })
                    .collect();
                let title = titles[i];
                let barchart = BarChart::default()
                    .data(BarGroup::default().bars(&bars))
                    .block(Block::new().title(format!("{title}")))
                    .bar_width(1)
                    .bar_gap(0)
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
