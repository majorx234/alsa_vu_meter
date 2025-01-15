use alsa::card::Card;
use alsa::card::Iter;
use alsa::ctl::DeviceIter;
use alsa::ctl::{CardInfo, Ctl};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, Error, ValueOr};
use itertools::Itertools;
use std::os::raw::c_int;

use crate::frontend::ProducerRbf32;
use ringbuf::traits::Producer;

#[derive(Debug)]
pub struct CardStuff {
    ctl_id_str: String,
    ctl_id: c_int,
    device: i32,
    card_id: String,
    card_name: String,
}

impl CardStuff {
    pub fn print(&self) {
        println!("{:?}", self);
    }
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
fn rms(buf: &[i16], channels: u32) -> (f32, f32) {
    assert_eq!(channels, 2);
    if buf.len() == 0 {
        return (0f32, 0f32);
    }
    let mut sum_left = 0f32;
    let mut sum_right = 0f32;
    for (&x, &y) in buf.iter().tuples() {
        sum_left += (x as f32) * (x as f32);
        sum_right += (y as f32) * (y as f32);
    }
    let rms_left = (sum_left / (buf.len() as f32) / 2.0).sqrt();
    let rms_right = (sum_right / (buf.len() as f32) / 2.0).sqrt();

    // Convert value to decibels
    (
        20.0 * (rms_left / (i16::MAX as f32)).log10(),
        20.0 * (rms_right / (i16::MAX as f32)).log10(),
    )
}

fn read_loop(
    pcm: &PCM,
    mut ringbuffer_left_in: ProducerRbf32,
    mut ringbuffer_right_in: ProducerRbf32,
) -> Result<(), Error> {
    let io = pcm.io_i16()?;
    let mut buf = [0i16; 8192];
    let hwp = HwParams::any(&pcm)?;
    let channels = hwp.get_channels()?;
    loop {
        // Block while waiting for 8192 samples to be read from the device.
        assert_eq!(io.readi(&mut buf)?, buf.len());
        let (rms_left, rms_right) = rms(&buf, channels);
        // Todo put data in Ringbuffer
        println!("RMS: {:.1} dB, {:.1} dB", rms_left, rms_right);
        let _ = ringbuffer_left_in.try_push(rms_left);
        let _ = ringbuffer_right_in.try_push(rms_right);
    }
}

fn start_capture(
    pcm_device_name: &str,
    channel: u32,
    bit_rate: u32,
    format: Format,
) -> Result<PCM, Error> {
    let pcm = PCM::new(pcm_device_name, Direction::Capture, false)?;
    {
        // For this example, we assume 44100Hz, one channel, 16 bit audio.
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(channel)?;
        hwp.set_rate(bit_rate, ValueOr::Nearest)?;
        hwp.set_format(format)?;
        hwp.set_access(Access::RWInterleaved)?;
        pcm.hw_params(&hwp)?;
    }
    pcm.start()?;
    Ok(pcm)
}

pub fn create_capture_thread(
    ringbuffer_left_in: ProducerRbf32,
    ringbuffer_right_in: ProducerRbf32,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        // TODO: need to choose capturing device, "default" just for testing
        let channel = 2;
        let bit_rate = 48000;
        let format = Format::s16();
        let capture = start_capture("default", channel, bit_rate, format).unwrap();

        read_loop(&capture, ringbuffer_left_in, ringbuffer_right_in).unwrap();
    })
}
