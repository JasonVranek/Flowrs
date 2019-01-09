use crate::io::order::TradeType;
use crate::io::order::Order;

use std::sync::{Mutex, Arc};
use std::thread;
use std::thread::JoinHandle;

pub fn test_order_book_mod() {
	println!("Hello, order_book!");
}

pub struct Book {
	pub book_type: TradeType,
	pub orders: Mutex<Vec<Order>>,
	pub min_price: Mutex<f64>,
	pub max_price: Mutex<f64>,
}

impl Book {
    pub fn new(book_type: TradeType) -> Book {
    	Book {
    		book_type,
    		orders: Mutex::new(Vec::<Order>::new()),
    		min_price: Mutex::new(0.0),
    		max_price: Mutex::new(0.0),
    	}
    }

    pub fn add_order(&mut self, order: Order) {
    	let mut orders = self.orders.lock().unwrap();
    	orders.push(order);
    }

    // Blocking len() to acquire lock
    pub fn len(&mut self) -> usize {
    	let orders = self.orders.lock().unwrap();
    	orders.len()
    }
}


pub struct Queue {
    items: Mutex<Vec<Order>>,
}

impl Queue {
	pub fn new() -> Queue {
		Queue {
			items: Mutex::new(Vec::<Order>::new()),
		}
	}

	pub fn add(&self, order: Order) {
        let mut items = self.items.lock().unwrap();
        items.push(order);
	}

	pub fn pop(&self) -> Option<Order> {
		let mut items = self.items.lock().unwrap();
		items.pop()
	}
}

// Preprocess message in a new thread and append to queue
// order is the trader's order that this function takes ownership of
// queue is an Arc clone of the Queue stored on the heap
pub fn conc_recv_order(order: Order, queue: Arc<Queue>) -> JoinHandle<()> {
    thread::spawn(move || {
    	// The add function acquires the lock
    	queue.add(order);
    })

}


// Concurrently process orders in the queue. Each order is
// either of OrderType::{Enter, Update, Cancel}. Each order will
// modify the state of either the Bids or Asks Book, but must
// first acquire a lock on the respective book. 
pub fn conc_process_order_queue() {
    
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_book() {
		let book = Book::new(TradeType::Bid);
		assert_eq!(book.book_type, TradeType::Bid);
		assert_eq!(*book.min_price.lock().unwrap(), 0.0);
		assert_eq!(*book.max_price.lock().unwrap(), 0.0);
	}

	#[test]
	fn test_book_mutex() {
		// Make sure not to acquire another lock in the same scope or it will deadlock
		let book = Arc::new(Book::new(TradeType::Bid));
		let mut handles = Vec::new();
		{
			// spawn 10 threads to update the book
			for _ in 0..10 {
				// Create a threadsafe cloned reference to mutex
				let book = Arc::clone(&book);

				let handle = thread::spawn(move || {
					// Acquire lock and update book in separate thread
					let mut max_price = book.max_price.lock().unwrap();
					// dereference the mutex to modify
					*max_price += 5.0;
					// assert_eq!(*max_price, 5.0);
				});
				handles.push(handle);
			}
			
		}
		// Wait for all the threads to finish
		for handle in handles {
			handle.join().unwrap();
		}

		assert_eq!(*book.max_price.lock().unwrap(), 50.0);

	}
}























