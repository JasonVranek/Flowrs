extern crate flow_rs;
use flow_rs::exchange::order_processing::OrderProcessor;
use flow_rs::simulation::trader_behavior::*;
use flow_rs::exchange::queue::*;
use flow_rs::order::*;
use flow_rs::exchange::order_book::*;
use std::sync::Arc;
use rand::Rng;

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
		500.0,
		poly_clos_from_coef(vec![-3.0, 4.0]),
	)
}

pub fn setup_ask_order() -> Order {
	Order::new(
		String::from("ask_id"),
		OrderType::Enter,
		TradeType::Ask,
		0.0,
		100.0,
		500.0,
		poly_clos_from_coef(vec![-3.0, 4.0]),
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
		handles.push(OrderProcessor::conc_recv_order(order, Arc::clone(&queue)));
	}

	for h in handles {
		h.join().unwrap();
	}

	queue
}

pub fn rand_coef_vector() -> Vec<f64> {
	// Create a variable length vector filled with random f64's
	let mut rng = rand::thread_rng();
	let coefs: Vec<f64> = (0..rng.gen_range(0, 6)).map(|_| {
		let coef: f64 = rng.gen();
		coef * 10.0
	}).collect();
	coefs
}

pub fn n_bid_enters(n: u32) -> Vec<Order> {
	let mut bids = Vec::<Order>::new();
	for _ in 0..n {
		bids.push(rand_bid_enter());
	}
	bids
}

pub fn n_ask_enters(n: u32) -> Vec<Order> {
	let mut asks = Vec::<Order>::new();
	for _ in 0..n {
		asks.push(rand_ask_enter());
	}
	asks
}

pub fn setup_orders() -> (Vec<Order>, Vec<Order>) {
	let mut bids = Vec::<Order>::new();
	let mut asks = Vec::<Order>::new();
	for i in 0..100 {
		bids.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Bid, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_dem(i as f64, 100.0, 500.0),
		));
		asks.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Ask, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_sup(i as f64, 100.0, 500.0),
		));

	}

	(bids, asks)
}











