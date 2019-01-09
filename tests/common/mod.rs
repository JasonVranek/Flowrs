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

pub fn setup_bids_book() -> Book {
	Book::new(TradeType::Bid)
}

pub fn setup_asks_book() -> Book {
	Book::new(TradeType::Ask)
}

pub fn setup_queue() -> Queue {
	Queue::new()
}