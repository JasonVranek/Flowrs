use core::f64::MAX;
use crate::io::order::TradeType;
use crate::io::order::Order;
use crate::io::order::OrderType;

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
    		min_price: Mutex::new(MAX),
    		max_price: Mutex::new(0.0),
    	}
    }

    pub fn add_order(&self, order: Order) {
    	let mut orders = self.orders.lock().unwrap();
    	orders.push(order);
    	orders.sort_by(|a, b| a.p_high.partial_cmp(&b.p_high).unwrap());
    }

    // TODO make this less dumb
    pub fn update_order(&self, order: Order) {
    	// Acquire the lock
        let mut orders = self.orders.lock().unwrap();
        // Search for existing order's index
        let order_index = orders.iter().position(|o| o.trader_id == order.trader_id);

        if let Some(i) = order_index {
        	// Add new order to end of the vector
        	orders.push(order);
        	let last = orders.len() - 1;
        	orders.swap(i, last);
    		// Swap orders then pop off the old order that is now at the end of vector
        	orders.pop();
        } else {
        	println!("ERROR: order not found to update: {:?}", &order.trader_id)
        }
    }

    pub fn cancel_order(&self, order: Order) {
    	// Acquire the lock
        let mut orders = self.orders.lock().unwrap();
        // Search for existing order's index
        let order_index: Option<usize> = orders.iter().position(|o| &o.trader_id == &order.trader_id);

        if let Some(i) = order_index {
        	orders.remove(i);
        } else {
        	println!("ERROR: order not found to cancel: {:?}", &order.trader_id);
        }
    }

    pub fn peek_id_pos(&self, trader_id: String) -> Option<usize> {
    	// Acquire the lock
        let orders = self.orders.lock().unwrap();
        // Search for existing order's index
        orders.iter().position(|o| o.trader_id == trader_id)
    }

    pub fn update_max_price(&self, p_high: &f64) {
		let mut max_price = self.max_price.lock().unwrap();
		if *p_high > *max_price {
			*max_price = *p_high;
		} 
    }

	pub fn update_min_price(&self, p_low: &f64) {
		let mut min_price = self.min_price.lock().unwrap();
		if *p_low < *min_price {
			*min_price = *p_low;
		} 
    }

    // Blocking len() to acquire lock
    pub fn len(&self) -> usize {
    	let orders = self.orders.lock().unwrap();
    	orders.len()
    }

    pub fn get_min_price(&self) -> f64 {
    	let price = self.min_price.lock().unwrap();
    	price.clone() as f64
    }

    pub fn get_max_price(&self) -> f64 {
    	let price = self.max_price.lock().unwrap();
    	price.clone() as f64
    }

    pub fn find_new_max(&self) {
    	// find the order with the max price (from sorted list):
    	let orders = self.orders.lock().unwrap();

    	let new_max = orders.last().unwrap().p_high;

    	let mut max_price = self.max_price.lock().unwrap();
    	*max_price = new_max;
    }

    pub fn find_new_min(&self) {
    	// find the order with the min price
    	let orders = self.orders.lock().unwrap();
    	let start = MAX;
    	let new_min = orders.iter().fold(start, |min, order| if order.p_low < min {order.p_low} else {min});

    	let mut min_price = self.min_price.lock().unwrap();
    	*min_price = new_min;
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

	pub fn pop_all(&self) -> Vec<Order> {
		// Acquire the lock
		let mut items = self.items.lock().unwrap();
		// Pop all items out of the queue and return the contents as a vec
		items.drain(..).collect()
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
pub fn conc_process_order_queue(queue: Arc<Queue>, 
								bids: Arc<Book>, 
								asks: Arc<Book>) 
								-> Vec<JoinHandle<()>>{
	// Acquire lock of Queue
	// Pop off contents of Queue
	// match over the OrderType
	// process each order based on OrderType
	let mut handles = Vec::<JoinHandle<()>>::new();
	for order in queue.pop_all() {
		let handle = match order.trade_type {
			TradeType::Bid => {
				match order.order_type {
					OrderType::Enter => process_enter(order, Arc::clone(&bids)),
					OrderType::Update => process_update(order, Arc::clone(&bids)),
  	    			OrderType::Cancel => process_cancel(order, Arc::clone(&bids)),
				}
			}
			TradeType::Ask => {
				match order.order_type {
					OrderType::Enter => process_enter(order, Arc::clone(&asks)),
					OrderType::Update => process_update(order, Arc::clone(&asks)),
  	    			OrderType::Cancel => process_cancel(order, Arc::clone(&asks)),
				}
			}
		};
		handles.push(handle);
	}
	handles
}


// Adds the order to the Bids or Asks Book
pub fn process_enter(order: Order, book: Arc<Book>) -> JoinHandle<()> {
	// Spawn a new thread to process the order
    thread::spawn(move || {
    	// add_order acquires the lock on the book before mutating
    	book.update_max_price(&order.p_high);
    	book.update_min_price(&order.p_low);
    	book.add_order(order);
    })
}

// Updates an order in the Bids or Asks Book
pub fn process_update(order: Order, book: Arc<Book>) -> JoinHandle<()> {
	// update min/max if this overwrites current min/max OR this order contains new min/max
	// ..
    // Spawn a new thread to update
    thread::spawn(move || {
    	book.update_order(order);
    })
}

// Cancels the order living in the Bids or Asks Book
pub fn process_cancel(order: Order, book: Arc<Book>) -> JoinHandle<()> {
	
    // Spawn a new thread to cancel and enter
    thread::spawn(move || {
		let p_high = order.p_high.clone();
		let p_low = order.p_low.clone();

		book.cancel_order(order);

		// update min/max if we just cancelled previous min/max
		if p_high == book.get_max_price() {
			book.find_new_max();
		}
		if p_low == book.get_min_price() && p_low != 0.0 {
			book.find_new_min();
			println!("Cancelling old min price");
		}
    	
    })
}




#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_book() {
		let book = Book::new(TradeType::Bid);
		assert_eq!(book.book_type, TradeType::Bid);
		assert_eq!(*book.min_price.lock().unwrap(), MAX);
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























