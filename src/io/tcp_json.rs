use crate::exchange::order_processing::JsonOrder;
use crate::exchange::queue::Queue;
use crate::controller::Task;

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::sync::Arc;

/// A simple tcp server that listens for incoming messages asynchronously. Each message
/// is parsed from a JSON into the internal Order type used in the exchange. This function
/// returns an AsnycTask to be used by the Controller module running Tokio.
pub fn tcp_listener(queue: Arc<Queue>, address: String) -> Task { 
	 // Bind a TcpListener to a local port
	let addr = address.parse().unwrap();
	let listener = TcpListener::bind(&addr).unwrap();

    println!("Running server on {}", addr);

	// start a tcp server that accepts JSON objects 
	let tcp_server = listener.incoming().for_each(move |socket| {
		// Clone the queue into the closure
		let queue = Arc::clone(&queue);

		// Deserialize the stream from the socket
        let deserialized = JsonOrder::deserialize(socket).map_err(|e| println!("ERR: {:?}", e));

        // Spawn a task that converts JSON to an Order and adds to queue
        tokio::spawn(deserialized.for_each(move |msg| {
            JsonOrder::process_new(msg, Arc::clone(&queue));
            Ok(())
        }));

        Ok(())
    })
    .map_err(|_| ());

    Task {
        task: Box::new(tcp_server),
    }
}


/// Creates an asynchronous task that opens a TCP connection and sends a JSON order
pub fn tcp_send_json(json: serde_json::Value, address: String) -> Task{
    // let (t_id, ot, tt, pl, ph, u) = order_params;
    // Creates a JSON from a reference of an order and sends it over TCP
    let addr = address.parse().unwrap();

    let client = TcpStream::connect(&addr).and_then(move |socket| {
        // Make a new json writer
        let serialized = JsonOrder::serializer(socket);

        // Send the value
        serialized
            .send(json).map(|_| ())
        }).map_err(|_| ());

    Task {
        task: Box::new(client),
    }
}





















