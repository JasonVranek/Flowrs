use crate::order::Order;


use std::collections::HashMap;
use std::sync::Mutex;

/// A struct for keeping track of the orders sent to the exchange. The
/// struct is a threadsafe HashMap that stores the raw order that each trader
/// has generated, indexed by their unique trader_id. This is used externally
/// to simulate traders communicating with the exchange.
pub struct Traders {
	pub traders: Mutex<HashMap<String, Order>>,
}

impl Traders {
	pub fn new() -> Self {
		Traders {
			traders: Mutex::new(HashMap::new()),
		}
	}

	/// Add a new order to the Traders HashMap
	pub fn new_trader(&mut self, order: Order) {
		let mut traders = self.traders.lock().unwrap();
		// or_insert will not overwrite an existing entry, but will insert if the key doesn't exist
		traders.entry(order.trader_id.clone()).or_insert(order);
	}

	/// Add a vector of new orders to the Traders HashMap. This is preferable to new_trader
	/// as the mutex lock only has to be acquired once.
	pub fn new_traders(&self, orders: Vec<Order>) {
		let mut traders = self.traders.lock().unwrap();
		for order in orders {
			traders.entry(order.trader_id.clone()).or_insert(order);
		}
	}

	/// Updates a trader's order in the HashMap with the supplied 'order'
	pub fn update_trader(&mut self, order: Order) {
		self.traders.lock().unwrap().insert(order.trader_id.clone(), order);
	}

	/// Removes the trader and their order from the HashMap
	pub fn del_trader(&mut self, trader_id: String) {
		self.traders.lock().unwrap().remove(&trader_id);
	}

	/// Utility function for seeing how many Trader's are currently active
	pub fn num_traders(&self) -> usize {
		self.traders.lock().unwrap().len()
	}
}
















