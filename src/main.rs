extern crate flow_rs;

use flow_rs::State;
use std::sync::{Mutex, Arc};

use flow_rs::io::order::*;
use flow_rs::io::order;
use flow_rs::io::trader;
use flow_rs::exchange::order_book::*;
use flow_rs::exchange::auction;

use tokio::prelude::*;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use futures::future::lazy;


fn main() {
    start_flow_market();
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

// Function to be called within tokio::run()
// Box<Future<Item = (), Error = ()> + Send>
fn start_flow_market() {
	// start listener for tcp connections
	// let (queue, bids_book, asks_book, state) = flow_rs::setup();
    
    let (queue, bids_book, asks_book, state) = fill_book();

    // Bind a TcpListener to a local port
	// let addr = "127.0.0.1:6142".parse().unwrap();
	// let listener = TcpListener::bind(&addr).unwrap();

	let batch_interval = 100;
    
	// spawn tasks to process the order queue and schedule the auction
	let auction_task = flow_rs::auction_interval(Arc::clone(&bids_book), 
		                          Arc::clone(&asks_book), 
		                          Arc::clone(&state), batch_interval);

	let queue_task = flow_rs::process_queue_interval(Arc::clone(&queue), 
		                                             Arc::clone(&bids_book), 
		                                             Arc::clone(&asks_book),
		                                             Arc::clone(&state));

	tokio::run(auction_task.join(queue_task).map(|_| ()));
}















