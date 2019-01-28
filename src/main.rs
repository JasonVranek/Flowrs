extern crate flow_rs;
extern crate tokio;

use flow_rs::io::ws_json::ws_listener;
use flow_rs::io::tcp_json::tcp_listener;
use flow_rs::exchange::queue_processing::QueueProcessor;
use flow_rs::exchange::auction::Auction;
use flow_rs::controller::Controller;

use std::sync::Arc;



fn main() {
	// Initialize the Exchange
	let (queue, bids_book, asks_book, state) = flow_rs::setup_exchange();

	// Create a new Controller to dispatch our tasks
	let mut controller = Controller::new();
    
	// create a task run an auction every batch_interval (milliseconds)
	let batch_interval = 3000;
	let auction_task = Auction::async_auction_task(Arc::clone(&bids_book), 
		                          Arc::clone(&asks_book), 
		                          Arc::clone(&state), batch_interval);
	controller.push(auction_task);

	// create a task that processes order queue every queue_interval (milliseconds)
	let queue_interval = 100;
	let queue_task = QueueProcessor::async_queue_task(Arc::clone(&queue), 
		                                             Arc::clone(&bids_book), 
		                                             Arc::clone(&asks_book),
		                                             Arc::clone(&state),
		                                             queue_interval);
	controller.push(queue_task);

	// Spawn the tcp server task that listens for incoming orders in JSON format
	let tcp_server = tcp_listener(Arc::clone(&queue), format!("127.0.0.1:5000"));
	controller.push(tcp_server);


	// Spawn the websocket server thread that listens for incoming orders in JSON format
	let address: &'static str = "127.0.0.1:3015";
	let _ws_server = ws_listener(Arc::clone(&queue), &address);
	
	// Loop forever asynchronously running tasks
	controller.run();
}









