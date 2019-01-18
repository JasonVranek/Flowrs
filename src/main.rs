extern crate flow_rs;
extern crate futures;
extern crate serde_json;
extern crate tokio;
extern crate tokio_serde_json;

use flow_rs::State;
use std::sync::{Mutex, Arc};

use flow_rs::exchange::order_book::*;

// use tokio::prelude::*;
use tokio::net::{TcpListener};
use futures::{Future, Stream};
use tokio::codec::{FramedRead, LengthDelimitedCodec};
use serde_json::Value;
use tokio_serde_json::ReadJson;


fn main() {
	// start listener for tcp connections
	let (queue, bids_book, asks_book, state) = flow_rs::setup();

    // let (queue, bids_book, asks_book, state) = fill_book();

    // Bind a TcpListener to a local port
	let addr = "127.0.0.1:6142".parse().unwrap();
	let listener = TcpListener::bind(&addr).unwrap();

    
	// spawn task run an auction every batch_interval (milliseconds)
	let batch_interval = 3000;
	let auction_task = flow_rs::auction_interval(Arc::clone(&bids_book), 
		                          Arc::clone(&asks_book), 
		                          Arc::clone(&state), batch_interval);

	// spawn task that processes order queue every queue_interval (milliseconds)
	let queue_interval = 100;
	let queue_task = flow_rs::process_queue_interval(Arc::clone(&queue), 
		                                             Arc::clone(&bids_book), 
		                                             Arc::clone(&asks_book),
		                                             Arc::clone(&state),
		                                             queue_interval);

	// start a tcp server that accepts JSON objects 
	let server = listener.incoming().for_each(move |socket| {
		// Clone the queue into the closure
		let queue = Arc::clone(&queue);

		// Delimit frames using a length header
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());

        // Deserialize frames
        let deserialized = ReadJson::<_, Value>::new(length_delimited)
            .map_err(|e| println!("ERR: {:?}", e));

        // Spawn a task that converts JSON to an Order and adds to queue
        tokio::spawn(deserialized.for_each(move |msg| {
            // println!("GOT: {:?} @ {:?}", msg, flow_rs::get_time());
            flow_rs::process_new(msg, Arc::clone(&queue));
            Ok(())
        }));

        Ok(())
    })
    .map_err(|_| ());


	println!("Running server on localhost:6142");

	// Use join/join_all to combine futures into a single future to use in tokio::run
	tokio::run(server.join(queue_task).map(|_| ())
		.join(auction_task).map(|_| ()));
}


fn fill_book() -> (Arc<Queue>, Arc<Book>, Arc<Book>, Arc<Mutex<State>>) {
	// Initialize 
    let (queue, bids_book, asks_book, state) = flow_rs::setup();
    
    let (bids, asks) = flow_rs::setup_orders();
	let mut handles = Vec::new();

	// Send all the orders in parallel 
	for bid in bids {
		handles.push(conc_recv_order(bid, Arc::clone(&queue)));
	}
	for ask in asks {
		handles.push(conc_recv_order(ask, Arc::clone(&queue)));
	}

	// Wait for the threads to finish
	for h in handles {
		h.join().unwrap();
	}

	// Process all of the orders in the queue
	let handles = conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));

	for h in handles {
		h.join().unwrap();
	}

	assert_eq!(bids_book.len(), 100);
	assert_eq!(asks_book.len(), 100);

	// let cross_price = auction::bs_cross(Arc::clone(&bids_book), Arc::clone(&asks_book)).unwrap();
	// assert_eq!(cross_price, 81.09048166081236);

	(queue, bids_book, asks_book, state)
}















