use crate::order::Order;
use std::sync::Mutex;

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