use alsa::card::Card;
use alsa::card::Iter;
use alsa::ctl::DeviceIter;
use alsa::ctl::{CardInfo, Ctl};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, Error, ValueOr};
use std::os::raw::c_int;

use crate::frontend::ProducerRbf32;

struct CardStuff {
    ctl_id_str: String,
    ctl_id: c_int,
    device: i32,
    card_id: String,
    card_name: String,
}

fn card_info(card: &Card) -> Result<Vec<CardStuff>, Error> {
    let ctl_id = format!("hw:{}", card.get_index());
    let ctl = Ctl::new(&ctl_id, false)?;
    let cardinfo = ctl.card_info()?;
    let card_id = cardinfo.get_id()?;
    let card_name = cardinfo.get_name()?;
    let mut card_stuff = Vec::new();
    for device in DeviceIter::new(&ctl) {
        // Read info from Ctl
        let pcm_info = ctl.pcm_info(device as u32, 0, Direction::Capture)?;
        let pcm_name = pcm_info.get_name()?.to_string();
        let card_stuff_item = CardStuff {
            ctl_id_str: ctl_id.clone(),
            ctl_id: card.get_index(),
            device,
            card_id: card_id.to_string(),
            card_name: card_name.to_string(),
        };
        card_stuff.push(card_stuff_item);
        println!(
            "card: {:<2} id: {:<10} device: {:<2} card name: '{}' PCM name: '{}'",
            card.get_index(),
            card_id,
            device,
            card_name,
            pcm_name
        );
    }
    Ok(card_stuff)
}

pub fn get_alsa_cards() -> Vec<Vec<CardStuff>> {
    let mut cards_stuff = Vec::new();
    let cards = Iter::new();
    cards.for_each(|card| {
        if let Ok(c) = card {
            let card_stuff = card_info(&c).unwrap_or_default();
            cards_stuff.push(card_stuff);
        }
    });
    return cards_stuff;
}

// Calculates RMS (root mean square) as a way to determine volume
fn rms(buf: &[i16]) -> f64 {
    if buf.len() == 0 {
        return 0f64;
    }
    let mut sum = 0f64;
    for &x in buf {
        sum += (x as f64) * (x as f64);
    }
    let r = (sum / (buf.len() as f64)).sqrt();
    // Convert value to decibels
    20.0 * (r / (i16::MAX as f64)).log10()
}

fn read_loop(
    pcm: &PCM,
    mut ringbuffer_left_in: ProducerRbf32,
    mut ringbuffer_right_in: ProducerRbf32,
) -> Result<(), Error> {
    let io = pcm.io_i16()?;
    let mut buf = [0i16; 8192];
    loop {
        // Block while waiting for 8192 samples to be read from the device.
        assert_eq!(io.readi(&mut buf)?, buf.len());
        let r = rms(&buf);
        // Todo put data in Ringbuffer
        println!("RMS: {:.1} dB", r);
    }
}

fn start_capture(pcm_device_name: &str) -> Result<PCM, Error> {
    let pcm = PCM::new(pcm_device_name, Direction::Capture, false)?;
    {
        // For this example, we assume 44100Hz, one channel, 16 bit audio.
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(1)?;
        hwp.set_rate(44100, ValueOr::Nearest)?;
        hwp.set_format(Format::s16())?;
        hwp.set_access(Access::RWInterleaved)?;
        pcm.hw_params(&hwp)?;
    }
    pcm.start()?;
    Ok(pcm)
}

pub fn create_capture_thread(
    mut ringbuffer_left_in: ProducerRbf32,
    mut ringbuffer_right_in: ProducerRbf32,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        // TODO: need to choose capturing device, "default" just for testing
        let capture = start_capture("default").unwrap();

        read_loop(&capture, ringbuffer_left_in, ringbuffer_right_in).unwrap();
    })
}
