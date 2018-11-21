use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crayon::errors::Result;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, AudioProcessingEvent};

use super::sampler::Sampler;
use super::Command;

const CHANNELS: u8 = 2;

pub fn run(rx: Arc<RwLock<Vec<Command>>>) -> Result<()> {
    info!("Create web audio mixer.",);

    let ctx = AudioContext::new().unwrap();

    let closure = Rc::new(RefCell::new(None));
    let clone = closure.clone();
    let mut sampler = Sampler::new(CHANNELS, ctx.sample_rate() as u32);

    let mut bufs = Vec::new();
    for _ in 0..CHANNELS {
        bufs.push(Vec::new());
    }

    *closure.borrow_mut() = Some(Closure::wrap(Box::new(move |e: AudioProcessingEvent| {
        if clone.borrow().is_some() {}

        {
            let mut rx = rx.write().unwrap();
            sampler.update(rx.drain(..));
        }

        // The output buffer contains the samples that will be modified and played
        let buffer = e.output_buffer().unwrap();

        let len = buffer.length();
        for buf in &mut bufs {
            buf.clear();
        }

        for _ in 0..len {
            for buf in &mut bufs {
                buf.push(sampler.sample());
            }
        }

        for (i, mut buf) in bufs.iter_mut().enumerate() {
            buffer.copy_to_channel(&mut buf, i as i32).unwrap();
        }
    }) as Box<FnMut(_)>));

    let source = ctx.create_buffer_source().unwrap();
    let processor = ctx.create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(0, CHANNELS as u32, CHANNELS as u32).unwrap();

    if let Some(closure) = closure.borrow().as_ref() {
        processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
    }

    source.connect_with_audio_node(&processor).unwrap();

    processor
        .connect_with_audio_node(&ctx.destination())
        .unwrap();

    source.start().unwrap();
    Ok(())
}
