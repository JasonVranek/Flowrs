use crate::order::Order;
use std::sync::Mutex;


/// A threadsafe FIFO queue to store unprocessed messages arriving from traders.
pub struct Queue {
    items: Mutex<Vec<Order>>,
}

impl Queue {
	pub fn new() -> Queue {
		Queue {
			items: Mutex::new(Vec::<Order>::new()),
		}
	}

	// New orders are pushed to the end of the Queue
	pub fn add(&self, order: Order) {
        let mut items = self.items.lock().unwrap();
        items.push(order);
	}

	pub fn pop(&self) -> Option<Order> {
		let mut items = self.items.lock().unwrap();
		items.pop()
	}

	// Empties the Queue into a vector of Orders. Drain() pops the items
	// out in the order of arrival, so once iterated upon, orders will be 
	// processed first -> last.
	pub fn pop_all(&self) -> Vec<Order> {
		// Acquire the lock
		let mut items = self.items.lock().unwrap();
		// Pop all items out of the queue and return the contents as a vec
		items.drain(..).collect()
	}
}