extern crate flow_rs;
use flow_rs::io::order::*;
use flow_rs::io::trader;
use flow_rs::exchange::order_book::*;
use flow_rs::exchange::auction;
use std::sync::{Mutex, Arc};
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
		handles.push(conc_recv_order(order, Arc::clone(&queue)));
	}

	for h in handles {
		h.join().unwrap();
	}

	queue
}

pub fn rand_ask_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	let coefs = rand_coef_vector();
	let id = gen_order_id();
	let u_max = gen_u_max();
	Order::new(
		id,
		OrderType::Enter,
		TradeType::Ask,
		p_l,
		p_h,
		u_max,
		poly_clos_from_coef(coefs),
	)
}

pub fn rand_bid_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	let coefs = rand_coef_vector();
	let id = gen_order_id();
	let u_max = gen_u_max();
	Order::new(
		id,
		OrderType::Enter,
		TradeType::Bid,
		p_l,
		p_h,
		u_max,
		poly_clos_from_coef(coefs),
	)
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

pub fn gen_prices() -> (f64, f64) {
	let mut rng = rand::thread_rng();
	let mut p_l: f64 = rng.gen();
	p_l *= 50.0;
	let mut p_h: f64 = rng.gen();
	p_h *= 100.0;
	while p_h < p_l {
		p_h = rng.gen();
		p_h *= 100.0;
	}
	(p_l, p_h)
}

pub fn gen_order_id() -> String {
	// Create a variable length vector filled with random chars
	let mut rng = rand::thread_rng();
	let mut id = String::new();
	for _ in 0..rng.gen_range(1, 10) {
		id.push(rand::random::<char>());
	}
	id
}

pub fn gen_u_max() -> f64 {
	let mut rng = rand::thread_rng();
	let u_max: f64 = rng.gen();
	u_max * 500.0
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











