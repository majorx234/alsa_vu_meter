use alsa::card::Card;
use alsa::card::Iter;
use alsa::ctl::DeviceIter;
use alsa::ctl::{CardInfo, Ctl};
use alsa::pcm::PCM;
use alsa::Direction;
use alsa::Error;

fn card_info(card: &Card) -> Result<(), Error> {
    let ctl_id = format!("hw:{}", card.get_index());
    let ctl = Ctl::new(&ctl_id, false)?;
    let cardinfo = ctl.card_info()?;
    let card_id = cardinfo.get_id()?;
    let card_name = cardinfo.get_name()?;
    for device in DeviceIter::new(&ctl) {
        // Read info from Ctl
        let pcm_info = ctl.pcm_info(device as u32, 0, Direction::Capture)?;
        let pcm_name = pcm_info.get_name()?.to_string();

        println!(
            "card: {:<2} id: {:<10} device: {:<2} card name: '{}' PCM name: '{}'",
            card.get_index(),
            card_id,
            device,
            card_name,
            pcm_name
        );
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let cards = Iter::new();
    cards.for_each(|card| {
        if let Ok(c) = card {
            card_info(&c).unwrap_or_default()
        }
    });
    Ok(())
}
