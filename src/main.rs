extern crate flow_rs;

use std::sync::{Mutex, Arc};

use flow_rs::io::order::*;
use flow_rs::io::order;
use flow_rs::io::trader;
use flow_rs::exchange::order_book::*;
use flow_rs::exchange::auction;

use tokio::prelude::*;
use tokio::io;
use tokio::timer::{Delay, Interval};
use std::time::{Duration, Instant, SystemTime};

fn main() {
    test();
}

fn test() {
	// Initialize 
    let (queue, bids_book, asks_book) = flow_rs::setup();
    
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

	let cross_price = auction::bs_cross(Arc::clone(&bids_book), Arc::clone(&asks_book)).unwrap();
	assert_eq!(cross_price, 81.09048166081236);

	let auction_task = auction_interval(Arc::clone(&bids_book), Arc::clone(&asks_book), 100);
	tokio::run(auction_task);

}

fn auction_interval(bids: Arc<Book>, asks: Arc<Book>, duration: u64) -> Box<Future<Item = (), Error = ()> + Send> {
	// let when = Instant::now() + Duration::from_millis(500);
	let task = Interval::new(Instant::now(),  Duration::from_millis(duration))
	    .for_each(move |_| {
	    	let now = SystemTime::now();
	    	println!("Starting Auction");
	    	let cross_price = auction::bs_cross(Arc::clone(&bids), Arc::clone(&asks)).unwrap();
	    	let done = now.elapsed().unwrap().subsec_nanos();
	    	println!("Found Cross at @{} \nP = {}", done, cross_price);
	    	Ok(())
	    })
	    .map_err(|e| panic!("Auction Delay error; err={:?}", e));

	Box::new(task)
}
// Function to be called within tokio::run()
fn start_flow_market() {
	// start listener for tcp connections

	// spawn a task to process the order queue and schedule the auction
}

// Make an enum that has the states of the flow market

enum FlowMarket {
	Processing,
	DeltaTime,
	Auction,
}















