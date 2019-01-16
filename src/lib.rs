use std::sync::Arc;
use rand::Rng;

use crate::exchange::order_book::*;
use crate::io::order::*;

pub mod io;
pub mod exchange;


pub fn setup() -> (Arc<Queue>, Arc<Book>, Arc<Book>) {
	let queue = Arc::new(Queue::new());
	let bids_book = Arc::new(Book::new(TradeType::Bid));
	let asks_book = Arc::new(Book::new(TradeType::Ask));
	(queue, bids_book, asks_book)
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

pub fn run_auction()  {

}