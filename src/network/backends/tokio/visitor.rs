use super::super::{Visitor};
use crate::errors::Result;
use tokio;
use futures::future::Future;
use futures::future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::ClientBuilder;
pub use websocket::OwnedMessage;
use websocket::message::CloseData;
use serde_json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[derive(Serialize,Deserialize)]
#[serde(tag = "connection_status", content = "c")]
pub enum ConnectionStatus {
    Error(ConnectionError),
    Ok,
}
#[derive(Serialize,Deserialize)]
pub enum ConnectionError {
    NotConnectedToInternet,
    CannotFindServer,
    InvalidDestination,
}
pub struct TokioVisitor {
    connection:String,
    proxy: (std::sync::mpsc::Sender<String>,std::sync::mpsc::Receiver<String>),
    server_proxy_tx: Arc<Mutex<mpsc::Sender<String>>>,
}
impl TokioVisitor{
    pub fn new() -> Result<Self>{
        let (ss_tx,_ss_rx) = mpsc::channel::<String>(3);
        let c = "".to_owned();
        Ok(TokioVisitor{
            connection:c,
            proxy: std::sync::mpsc::channel::<String>(),
            server_proxy_tx: Arc::new(Mutex::new(ss_tx))
        })
    }
}

impl Visitor for TokioVisitor{
    #[inline]
    fn create_connection(&mut self,param:String)->Result<()>{
        if self.connection != param{
            println!("param {:?}",param);
            self.connection = param.clone();
            let p = param.clone();
            let (tx, rx) = mpsc::channel(3);
            let mut ss_tx = self.server_proxy_tx.lock().unwrap();
            tx.clone().try_send("close".to_string()).unwrap();
            *ss_tx = tx;
            
            drop(ss_tx);
            let proxy_0  =self.proxy.0.clone();
            //let close = "close".to_stirng();
            
            std::thread::spawn(move|| {
                //std::thread::sleep(Duration::from_millis(500));
                let mut runtime = tokio::runtime::Builder::new().build().unwrap();
                let runner = ClientBuilder::new(&p).unwrap().add_protocol("rust-websocket")
                    .async_connect_insecure()
                    .join3(future::ok::<std::sync::mpsc::Sender<String>,websocket::result::WebSocketError>(proxy_0.clone()),
                    future::ok::<mpsc::Receiver<String>,websocket::result::WebSocketError>(rx))
                    .and_then(|((duplex, _), gui_c,rx)| {
                    let (to_server, from_server) = duplex.split();
                    let reader = from_server.for_each(move |msg| {
                        println!("msg {:?}",msg);
                        // ... convert it to a string for display in the GUI...
                        let _content = match msg {
                            OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                            OwnedMessage::Ping(d) => Some(OwnedMessage::Ping(d)),
                            OwnedMessage::Text(f) => {
                                gui_c.send(f).unwrap();
                                None
                            }
                            _ => None,
                        };
                        // ... and send that string _to_ the GUI.

                        Ok(())
                    });
                    
                let writer = rx
                .map_err(|()| unreachable!("rx can't fail"))
                .fold(to_server, |to_server, msg| {
                    let h= msg.clone();
                    if h =="close"{
                        to_server.send(OwnedMessage::Close(Some(CloseData{
                            status_code: 200,
                            reason: "Close".to_string()
                        })))
                    }else{
                        to_server.send(OwnedMessage::Text(h))
                    }
                }).map(|_|());
                // Use select to allow either the reading or writing half dropping to drop the other
                // half. The `map` and `map_err` here effectively force this drop.
                reader.select(writer).map(|_| ()).map_err(|(err, _)| err)
            });
            /*
            match runtime.block_on(runner) { //block_on
                Ok(_) => {
                    println!("connected");
                    let g = serde_json::to_string(&ConnectionStatus::Ok).unwrap();
                    proxy_0.clone().send(g).unwrap();
                }
                Err(_er) => {
                    println!("not connected");
                    let g = serde_json::to_string(&ConnectionStatus::Error(ConnectionError::CannotFindServer)).unwrap();
                    proxy_0.clone().send(g).unwrap();
                }
            }*/
            runtime.block_on(runner).unwrap();
            });
            dbg!("after");
        }
        Ok(())
    }

    #[inline]
    fn poll_events(&mut self, v: &mut Vec<String>) {
        while let Ok(s) = self.proxy.1.try_recv(){
            v.push(s);
        }
    }
    #[inline]
    fn send(&mut self,v:String){
        let j = self.server_proxy_tx.lock().unwrap();
        j.clone().send(v).wait().unwrap();
    }
}