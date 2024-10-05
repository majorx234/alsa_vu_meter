use std::os::raw::c_int;

use alsa::card::Card;
use alsa::card::Iter;
use alsa::ctl::DeviceIter;
use alsa::ctl::{CardInfo, Ctl};
use alsa::pcm::PCM;
use alsa::Direction;
use alsa::Error;

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

fn main() -> Result<(), Error> {
    let cards = Iter::new();
    let mut cards_stuff = Vec::new();
    cards.for_each(|card| {
        if let Ok(c) = card {
            let card_stuff = card_info(&c).unwrap_or_default();
            cards_stuff.push(card_stuff);
        }
    });
    Ok(())
}
