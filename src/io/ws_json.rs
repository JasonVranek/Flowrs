use crate::exchange::order_processing::JsonOrder;
use crate::exchange::queue::Queue;

use std::thread;
use std::sync::Arc;

use ws::{connect, listen, CloseCode, Sender, Handler, Message, Result, Handshake};
use serde_json;

// WebSocket handler
struct Server {
    out: Sender,
    queue: Arc<Queue>,
}

/// A simple websocket server that listens for incoming messages asynchronously. Each message
/// is parsed from a JSON into the internal Order type used in the exchange. 
impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // println!("Server got message '{}'. ", msg);

        // Clone the queue into the closure
		let queue = Arc::clone(&self.queue);

		// Consume websocket message converting to string
		if let Ok(text) = msg.into_text() {
			match serde_json::from_str::<serde_json::Value>(&text) {
				Ok(json) => {
		            JsonOrder::process_new(json, Arc::clone(&queue));
				},
				Err(e) => {
					println!("Could not parse JSON: {:?}", e);
				},
			}
		} 

        self.out.send("Parsed message!")
    }
}

pub fn ws_listener(queue: Arc<Queue>, addr: &'static str) -> thread::JoinHandle<()> { 
	env_logger::init();
    thread::spawn(move || {
    	listen(addr, |out| {
	         Server {
	         	out: out,
	         	queue: Arc::clone(&queue),
	         }
	    }).expect("Error with WS Server...");
    })
}



// A handler for the clients to establish websocket connections and
// react to the associated events such as on_open.
pub struct Client {
	out: Sender,
	json: serde_json::Value,
}

impl Handler for Client {
	fn on_open(&mut self, _: Handshake) -> Result<()> {
		let msg = serde_json::to_string(&self.json).unwrap();
		self.out.send(msg).expect("Error sending client msg");
		self.out.close(CloseCode::Normal)
	}
}

/// A simple websocket server that sends jsons. Each message
/// is parsed from a JSON into the internal Order type used in the exchange. 
pub fn ws_send_json(json: serde_json::Value, address: &'static str) {
	let _ws_client = connect(address, move |out| {
		Client { 
			out: out, 
			json: json.clone() 
		}
    }).expect("Error with Client");
}














