use crate::io::order::TradeType;
use crate::io::order::Order;

use std::sync::{Mutex, Arc};
use std::thread;

pub fn test_order_book_mod() {
	println!("Hello, order_book!");
}




pub struct Book {
	book_type: TradeType,
	orders: Box<Vec<Order>>,
	min_price: u32,
	max_price: u32,
}

impl Book {
    fn new(book_type: TradeType) -> Book {
    	Book {
    		book_type,
    		orders: Box::new(Vec::<Order>::new()),
    		min_price: 0,
    		max_price: 0,
    	}
    }
}

// Preprocess message and append to queue
pub fn concurrent_receive_order() {

}


// Concurrently process orders in the queue. Each order is
// either of OrderType::{Enter, Update, Cancel}. Each order will
// modify the state of either the Bids or Asks Book, but must
// first acquire a lock on the respective book. 
pub fn concurrent_process_order_queue() {

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_book() {
		let book = Book::new(TradeType::Bid);
		assert_eq!(book.book_type, TradeType::Bid);
		assert_eq!(book.min_price, 0);
		assert_eq!(book.max_price, 0);
	}
}























