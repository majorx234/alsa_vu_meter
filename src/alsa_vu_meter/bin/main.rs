use ringbuf::{traits::Split, HeapRb};
use std::io::Error;
use std::thread::sleep;
use std::time::Duration;
mod frontend;
use crate::frontend::create_gui_thread;
mod alsa_pcm_stream;
use alsa_pcm_stream::{create_capture_thread, get_alsa_cards};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// audio device name
    #[arg(short, long)]
    device: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let devices = if let Some(device) = args.device {
        vec![device]
    } else {
        let mut devices = Vec::<String>::new();
        let card_stuffs = get_alsa_cards();
        for cards in card_stuffs {
            for card in cards {
                devices.push(format!("{:?}", card));
                card.print();
            }
        }
        devices
    };
    let ringbuffer_left = HeapRb::<f32>::new(96000);
    let ringbuffer_right = HeapRb::<f32>::new(96000);

    let (ringbuffer_left_in, ringbuffer_left_out) = ringbuffer_left.split();
    let (ringbuffer_right_in, ringbuffer_right_out) = ringbuffer_right.split();

    // TODO: create thread here and send data ro vu meter via channel
    let capture_thread = create_capture_thread(ringbuffer_left_in, ringbuffer_right_in);
    let mut run = true;

    let tui_thread = create_gui_thread(ringbuffer_left_out, ringbuffer_right_out, devices);
    println!("start TUI");
    let sleep_time = Duration::from_millis(500);
    while run {
        sleep(sleep_time);
    }

    println!("stop vu meter");
    let _ = capture_thread.join();
    let _ = tui_thread.join();
    Ok(())
}
