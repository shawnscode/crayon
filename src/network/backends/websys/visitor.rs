use super::super::{Visitor};
use wasm_bindgen::prelude::*;
use crate::errors::Result;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
#[allow(dead_code)]
pub struct WebVisitor {
    connections:Vec<String>,
    events: Arc<Mutex<Vec<String>>>,
    on_message: Closure<dyn std::ops::FnMut(web_sys::MessageEvent)>,
    ws: Option<WebSocket>
}

impl WebVisitor{
    pub fn new() -> Result<Self>{
        let events = Arc::new(Mutex::new(Vec::new()));
        let on_message = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |evt: MessageEvent| {
                let response = evt.data()
                            .as_string()
                .expect("Can't convert received data to a string");
                console_log!("message event, received data: {:?}", response);
                clone.lock().unwrap().push(response);
            }) as Box<dyn FnMut(MessageEvent)>)
        };
        Ok(WebVisitor{
            connections: Vec::new(),
            events: events,
            on_message:on_message,
            ws: None
        })
    }
}

impl Visitor for WebVisitor{
    #[inline]
    fn create_connection(&mut self,param:String)->Result<()>{
        if !self.connections.contains(&param){
        //let ws = WebSocket::new_with_str_sequence(&param,&JsValue::from_str("rust-websocket")).unwrap();
        let ws = WebSocket::new(&param).unwrap();
        ws.set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
        //self.on_message.forget();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            console_log!("error event: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        self.ws = Some(ws);
        self.connections.push(param);
        }
        Ok(())
    }
    #[inline]
    fn poll_events(&mut self, v: &mut Vec<String>) {
        let mut events = self.events.lock().unwrap();
        v.extend(events.drain(..));
    }
    #[inline]
    fn send(&mut self,v:String){
        if let Some(ws) = &self.ws{
            ws.send_with_str(&v).unwrap();
        }
    }
}