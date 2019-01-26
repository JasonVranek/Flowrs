pub mod io;
pub mod exchange;
pub mod simulation;
pub mod order;
pub mod controller;
pub mod utility;

use crate::exchange::order_book::Book;
use crate::order::TradeType;
use crate::exchange::queue::Queue;
use crate::controller::State;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

use std::sync::{Mutex, Arc};


pub fn setup_exchange() -> (Arc<Queue>, Arc<Book>, Arc<Book>, Arc<Mutex<State>>) {
	let queue = Arc::new(Queue::new());
	let bids_book = Arc::new(Book::new(TradeType::Bid));
	let asks_book = Arc::new(Book::new(TradeType::Ask));
	(queue, bids_book, asks_book, Arc::new(Mutex::new(State::Process)))
}

















