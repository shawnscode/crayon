use std::sync::{Arc, RwLock};
use std::thread::Builder;

use cpal::{self, EventLoop, StreamData, UnknownTypeOutputBuffer};
use crayon::errors::Result;

use super::sampler::Sampler;
use super::Command;

pub fn run(rx: Arc<RwLock<Vec<Command>>>) -> Result<()> {
    let device = cpal::default_output_device()
        .ok_or_else(|| format_err!("No avaiable audio output device"))?;

    let format = device
        .default_output_format()
        .expect("The device doesn't support any format.");

    let events = EventLoop::new();
    let stream = events.build_output_stream(&device, &format).unwrap();

    info!(
        "Create audio mixer based on CPAL. [{:?}] {:?}.",
        device.name(),
        format
    );

    let mut sampler = Sampler::new(format.channels as u8, format.sample_rate.0 as u32);
    Builder::new()
        .name("Audio".into())
        .spawn(move || {
            let mut bufs = Vec::new();

            events.run(move |id, buffer| {
                if stream != id {
                    return;
                }

                {
                    let mut rx = rx.write().unwrap();
                    ::std::mem::swap(&mut bufs, &mut rx);
                }

                sampler.update(bufs.drain(..));

                if let StreamData::Output { buffer } = buffer {
                    match buffer {
                        UnknownTypeOutputBuffer::U16(mut buffer) => {
                            for v in buffer.iter_mut() {
                                *v = sampler.sample_u16();
                            }
                        }
                        UnknownTypeOutputBuffer::I16(mut buffer) => {
                            for v in buffer.iter_mut() {
                                *v = sampler.sample_i16();
                            }
                        }
                        UnknownTypeOutputBuffer::F32(mut buffer) => {
                            for v in buffer.iter_mut() {
                                *v = sampler.sample();
                            }
                        }
                    }
                }
            })
        }).expect("Failed to create thread for `AudioSystem`.");

    Ok(())
}
