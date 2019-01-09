// extern crate <name_of_my_crate_to_test>
use std::sync::{Mutex, Arc};
use flow_rs::io::order::*;
use flow_rs::io::trader;
use flow_rs::exchange::order_book::*;
use flow_rs::exchange::auction;

// Include the common module for setting up state for tests
mod common;



#[test]
fn default_test() {
	common::setup();
	assert_eq!(1, 1);
}

#[test]
fn test_add_order_to_book() {
	let bid = common::setup_bid_order();

	let mut book = common::setup_bids_book();

	book.add_order(bid);

	assert_eq!(book.len(), 1);

	let mut order = book.orders.lock().unwrap().pop().unwrap();

	// The default closure: -3x + 4
	assert_eq!(order.calculate(5.0), -11.0);
}


#[test]
fn test_conc_queue_recv_order() {
	// Setup a queue
	let queue = Arc::new(common::setup_queue());

	let mut order = common::setup_bid_order();

	// Mutate order
	order.p_high = 199.0;

	// Accept order in a new thread
	let handle = conc_recv_order(order, Arc::clone(&queue));

	// Wait for thread to finish
	handle.join().unwrap();

	// Confirm the queue's order is correct
	let order = queue.pop().unwrap();

	assert_eq!(order.p_high, 199.0);
}

#[test]
fn test_queue_pop_all() {
	let queue = common::setup_full_queue();
	let popped_off = queue.pop_all();
	assert_eq!(popped_off[0].order_type, OrderType::Enter);
	assert_eq!(popped_off[1].order_type, OrderType::Update);
	assert_eq!(popped_off[2].order_type, OrderType::Cancel)
}


// #[test]
// fn test_conc_process_order_queue() {
// 	let queue = common::setup_full_queue();
// }



















