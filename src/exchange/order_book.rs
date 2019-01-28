use core::f64::MAX;
use crate::order::{Order, TradeType};

use std::sync::Mutex;
use std::io;

pub fn test_order_book_mod() {
	println!("Hello, order_book!");
}

/// The struct for the order books in the exchange. The purpose
/// is to keep track of bids and asks for calculating the aggregate
/// supply and demand to find the market clearing price. 
/// book_type: TradeType{Bid, Ask} -> To differentiate the two order books
/// orders: Mutex<Vec<Order>> -> Threadsafe vector to keep track of orders
/// min_price: Mutex<f64> -> Threadsafe minimum market price for computing clearing price
/// max_price: Mutex<f64> -> Threadsafe maximum market price for computing clearing price
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

    /// Adds a new order to the Book after acquiring a lock, then sorts by p_high
    pub fn add_order(&self, order: Order) -> io::Result<()> {
    	let mut orders = self.orders.lock().expect("ERROR: Couldn't lock book to update order");
    	orders.push(order);
    	orders.sort_by(|a, b| a.p_high.partial_cmp(&b.p_high).unwrap());
    	Ok(())
    }

    /// Replaces the order in the order book with the supplied 'order' of the same trader_id
    pub fn update_order(&self, order: Order) -> Result<(), &'static str> {
    	// Acquire the lock
        let mut orders = self.orders.lock().expect("ERROR: Couldn't lock book to update order");
        // Search for existing order's index
        let order_index = orders.iter().position(|o| o.trader_id == order.trader_id);

        if let Some(i) = order_index {
        	// Add new order to end of the vector
        	orders.push(order);
    		// Swap orders then pop off the old order that is now at the end of vector
        	let last = orders.len() - 1;
        	orders.swap(i, last);
        	orders.pop();
        } else {
        	println!("ERROR: order not found to update: {:?}", &order.trader_id);
        	return Err("ERROR: order not found to update");
        }

        Ok(())
    }

    /// Cancels the existing order in the order book if it exists
    pub fn cancel_order(&self, order: Order) -> Result<(), &'static str> {
    	// Acquire the lock
        let mut orders = self.orders.lock().expect("couldn't acquire lock cancelling order");
        // Search for existing order's index
        let order_index: Option<usize> = orders.iter().position(|o| &o.trader_id == &order.trader_id);

        if let Some(i) = order_index {
        	orders.remove(i);
        } else {
        	println!("ERROR: order not found to cancel: {:?}", &order.trader_id);
        	return Err("ERROR: order not found to cancel");
        }

        Ok(())
    }

    pub fn peek_id_pos(&self, trader_id: String) -> Option<usize> {
    	// Acquire the lock
        let orders = self.orders.lock().unwrap();
        // Search for existing order's index
        orders.iter().position(|o| o.trader_id == trader_id)
    }

    /// Utility to see depth of order book
    pub fn len(&self) -> usize {
    	let orders = self.orders.lock().unwrap();
    	orders.len()
    }

    /// Atomically updates the Book's max price
    pub fn update_max_price(&self, p_high: &f64) {
		let mut max_price = self.max_price.lock().unwrap();
		if *p_high > *max_price {
			*max_price = *p_high;
		} 
    }

    /// Atomically updates the Book's min price
	pub fn update_min_price(&self, p_low: &f64) {
		let mut min_price = self.min_price.lock().unwrap();
		if *p_low < *min_price {
			*min_price = *p_low;
		} 
    }

    /// Returns the Book's min price
    pub fn get_min_price(&self) -> f64 {
    	let price = self.min_price.lock().unwrap();
    	price.clone() as f64
    }

    /// Returns the Book's max price
    pub fn get_max_price(&self) -> f64 {
    	let price = self.max_price.lock().unwrap();
    	price.clone() as f64
    }

    /// Finds a new maximum Book price in the event that the previous was
    /// updated or cancelled and updates the Book. Utilizes Book being sorted by p_high
    pub fn find_new_max(&self) {
    	// find the order with the max price (from sorted list):
    	let orders = self.orders.lock().unwrap();

    	let new_max = orders.last().unwrap().p_high;

    	// Update the book with new max price
    	let mut max_price = self.max_price.lock().unwrap();
    	*max_price = new_max;
    }

    /// Finds a new minimum Book price in the event that the previous was
    /// updated or cancelled and updates the Book.
    pub fn find_new_min(&self) {
    	let orders = self.orders.lock().unwrap();

    	// Iterates over all orders until a minimum is found
    	let new_min = orders.iter().fold(MAX, |min, order| if order.p_low < min {order.p_low} else {min});

    	// Update the book with new min price
    	let mut min_price = self.min_price.lock().unwrap();
    	*min_price = new_min;
    }
}


#[cfg(test)]
mod tests {
	use super::*;
    use crate::order::{TradeType};
    use std::sync::Arc;
    use std::thread;

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























