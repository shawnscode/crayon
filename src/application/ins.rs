use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::errors::*;
use byteorder::{LittleEndian, WriteBytesExt};

pub const PORT: u32 = 9338;

pub trait Inspectable: Send + Sync {
    fn inspect(&self, buf: &mut Vec<u8>);
}

pub struct InspectSystem {
    listener: TcpListener,
    ins: HashMap<String, Arc<Inspectable>>,
}

impl InspectSystem {
    pub fn new(port: u32) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        InspectSystem {
            listener,
            ins: HashMap::new(),
        }
    }

    pub fn attach<T: Inspectable + 'static>(&mut self, id: &str, sys: Arc<T>) {
        self.ins.insert(id.to_string(), sys);
    }

    pub fn listen(mut self) {
        info!(
            "[InspectSystem] Starting inspector at {}:{}",
            "127.0.0.1", PORT
        );

        thread::Builder::new()
            .name("Inspector".to_string())
            .spawn(move || loop {
                match self.connect() {
                    Ok((stream, addr)) => {
                        if let Err(err) = self.worker(stream) {
                            warn!("[InspectSystem] Connection with {} broken! {}.", addr, err);
                        }
                    }
                    Err(err) => {
                        warn!("[InspectSystem] Failed to accept connection! {}", err);
                    }
                }

                ::std::thread::sleep(Duration::from_secs(1));
            })
            .unwrap();
    }

    fn connect(&mut self) -> Result<(TcpStream, SocketAddr)> {
        let (stream, addr) = self.listener.accept()?;
        info!("[InspectSystem] Accepting watcher from {}.", addr);
        Ok((stream, addr))
    }

    fn worker(&mut self, mut stream: TcpStream) -> Result<()> {
        let mut buf = Vec::new();
        let mut element = Vec::new();

        loop {
            buf.clear();
            element.clear();

            for (k, v) in &self.ins {
                buf.write_u8(k.len() as u8)?;
                buf.extend_from_slice(k.as_bytes());

                v.inspect(&mut element);

                buf.write_u32::<LittleEndian>(element.len() as u32)?;
                buf.append(&mut element);
            }

            stream.write_all(&buf)?;
            ::std::thread::sleep(Duration::from_secs(1));
        }
    }
}
