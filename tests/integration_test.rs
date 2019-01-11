// extern crate <name_of_my_crate_to_test>
use std::sync::{Mutex, Arc};
use std::thread::sleep;
use std::time::Duration;
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
	assert_eq!(popped_off.len(), 3);
}

#[test]
fn test_process_queue() {
	// Setup queue and order books
	let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let (bids, asks) = common::setup_orders();
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

	let b_max_price = bids_book.get_max_price();
	let b_min_price = bids_book.get_min_price();
	let a_max_price = asks_book.get_max_price();
	let a_min_price = asks_book.get_min_price();

	assert_eq!(b_max_price, 100.0);
	assert_eq!(b_min_price, 0.0);
	assert_eq!(a_max_price, 100.0);
	assert_eq!(a_min_price, 0.0);

	let (min, max) = auction::get_price_bounds(Arc::clone(&bids_book), Arc::clone(&asks_book));
	assert_eq!(min, 0.0);
	assert_eq!(max, 100.0);

}

#[test]
pub fn test_find_crossing_price() {
    let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let (bids, asks) = common::setup_orders();
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
}




















