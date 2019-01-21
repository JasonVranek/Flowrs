extern crate flow_rs;
extern crate futures;
extern crate tokio;

use flow_rs::io::tcp_json::tcp_listener;
use flow_rs::exchange::queue_processing::QueueProcessor;
use flow_rs::exchange::auction::Auction;

use futures::Future;
use std::sync::Arc;
use futures::future::{join_all};


fn main() {
	// start listener for tcp connections
	let (queue, bids_book, asks_book, state) = flow_rs::setup_exchange();

	let mut tasks = Vec::new();
    
	// spawn task run an auction every batch_interval (milliseconds)
	let batch_interval = 3000;
	let auction_task = Auction::async_auction_task(Arc::clone(&bids_book), 
		                          Arc::clone(&asks_book), 
		                          Arc::clone(&state), batch_interval);

	// spawn task that processes order queue every queue_interval (milliseconds)
	let queue_interval = 100;
	let queue_task = QueueProcessor::async_queue_task(Arc::clone(&queue), 
		                                             Arc::clone(&bids_book), 
		                                             Arc::clone(&asks_book),
		                                             Arc::clone(&state),
		                                             queue_interval);

	// Spawn the tcp server task that listens for incoming orders in JSON format
	let tcp_server = tcp_listener(Arc::clone(&queue), format!("127.0.0.1:6142"));

	tasks.push(auction_task);
	tasks.push(queue_task);
	tasks.push(tcp_server);
	
	// Use join/join_all to combine futures into a single future to use in tokio::run
	// tokio::run(tcp_server.join(queue_task).map(|_| ())
	// 	.join(auction_task).map(|_| ()));
	tokio::run(join_all(tasks).map(|_| ()));
}









