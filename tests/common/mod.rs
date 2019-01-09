extern crate flow_rs;
use flow_rs::io::order::*;
use flow_rs::io::trader;
use flow_rs::exchange::order_book::*;
use flow_rs::exchange::auction;
use std::sync::{Mutex, Arc};

pub fn setup() {
	// setup code specific to lib's tests go here
	// this code can then be accessed from other tests via
	// common::setup()
}

pub fn setup_bid_order() -> Order {
	Order::new(
		String::from("bid_id"),
		OrderType::Enter,
		TradeType::Bid,
		0.0,
		100.0,
		poly_clos_from_coef(&[-3.0, 4.0]),
	)
}

pub fn setup_ask_order() -> Order {
	Order::new(
		String::from("ask_id"),
		OrderType::Enter,
		TradeType::Ask,
		0.0,
		100.0,
		poly_clos_from_coef(&[-3.0, 4.0]),
	)
}

pub fn setup_bids_book() -> Book {
	Book::new(TradeType::Bid)
}

pub fn setup_asks_book() -> Book {
	Book::new(TradeType::Ask)
}

pub fn setup_queue() -> Queue {
	Queue::new()
}

pub fn each_order_type() -> Vec<Order> {
	let mut orders = Vec::<Order>::new();

	let b1 = setup_bid_order();
	orders.push(b1);
	let mut b2 = setup_bid_order();
	b2.order_type = OrderType::Update;
	orders.push(b2);
	let mut b3 = setup_bid_order();
	b3.order_type = OrderType::Cancel;
	orders.push(b3);
	orders
}

pub fn setup_full_queue() -> Arc<Queue> {
	let queue = Arc::new(setup_queue());
	let mut handles: Vec<_> = Vec::new();

	for order in each_order_type() {
		handles.push(conc_recv_order(order, Arc::clone(&queue)));
	}

	for h in handles {
		h.join().unwrap();
	}

	queue

}









