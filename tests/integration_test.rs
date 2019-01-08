// extern crate <name_of_my_crate_to_test>

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

