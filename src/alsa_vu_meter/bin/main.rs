use frontend::ConsumerRbf32;
use frontend::ProducerRbf32;
use ringbuf::{traits::Split, HeapRb};
use std::io::Error;
use std::thread::sleep;
use std::time::Duration;
mod frontend;
use crate::frontend::create_gui_thread;
mod alsa_pcm_stream;
use alsa_pcm_stream::create_capture_thread;

fn main() -> Result<(), Error> {
    let ringbuffer_left = HeapRb::<f32>::new(96000);
    let ringbuffer_right = HeapRb::<f32>::new(96000);

    let (ringbuffer_left_in, ringbuffer_left_out) = ringbuffer_left.split();
    let (ringbuffer_right_in, ringbuffer_right_out) = ringbuffer_right.split();

    // TODO: create thread here and send data for vu meeter via channel
    let capture_thread = create_capture_thread(ringbuffer_left_in, ringbuffer_right_in);
    let mut run = true;

    let tui_thread = create_gui_thread(ringbuffer_left_out, ringbuffer_right_out);
    while run {
        let sleep_time = Duration::from_millis(500);
        sleep(sleep_time);
    }

    let _ = capture_thread.join();
    let _ = tui_thread.join();
    Ok(())
}
