extern crate rand;
use crate::io::order::*;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use std::iter;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

/// A struct for keeping track of the orders sent to the exchange. The
/// struct is a threadsafe HashMap that stores the raw order that each trader
/// has generated, indexed by their unique trader_id. This is used externally
/// to simulate traders communicating with the exchange.
pub struct Traders {
	traders: Mutex<HashMap<String, Order>>,
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

/// Function for parsing an order into it's Json components. Workaround since
/// Box<Fn(f64) -> f64 + Send + Sync + 'static cannot implement clone trait
pub fn params_for_json(order: &Order) -> (String, OrderType, TradeType, f64, f64, f64) {
    return (order.trader_id.clone(),
        order.order_type.clone(),
        order.trade_type.clone(),
        order.p_low.clone(),
        order.p_high.clone(),
        order.u_max.clone());
}

/// A function to randomly generate update orders for existing traders within 
/// the Trader HashMap. The output is a vector of tuples where each tuple contains
/// the required parameters to generate a JSON formatted order. The supplied u32
/// 'upper' is to change the probability with which an update will occur for a 
/// given trader. Probability of update = (1 / upper), where upper > 0
pub fn gen_rand_updates(t_struct: Arc<Traders>, upper: u32) 
-> Vec<(String, OrderType, TradeType, f64, f64, f64)> 
{
		let mut rng = thread_rng();
		// Get a lock on the HashMap 
		let mut orders = t_struct.traders.lock().unwrap();

		// Vector of tuples to construct JSON messages
		let mut to_send: Vec<_> = Vec::new();

		// Iterate through hashmap and update based on rng
		for order in orders.values_mut() {
			// (1 / upper) chance of updating the given order
			if rng.gen_range(0, upper) == 1 {
				// generate a new order with same trader_id and trader_type
				let new_order = rand_update_order(order);
				// parse and save the new order for params to make JSON
				to_send.push(params_for_json(order));
				// save the new order in the hashmap
				*order = new_order;
			}
		}
		to_send
	}

/// A function to randomly generate cancel orders for existing traders within 
/// the Trader HashMap. The output is a vector of tuples where each tuple contains
/// the required parameters to generate a JSON formatted order. The supplied u32
/// 'upper' is to change the probability with which an update will occur for a 
/// given trader. Probability of update = (1 / upper), where upper > 0
pub fn gen_rand_cancels(t_struct: Arc<Traders>, upper: u32) 
-> Vec<(String, OrderType, TradeType, f64, f64, f64)> 
{
		let mut rng = thread_rng();
		// Get a lock on the HashMap 
		let mut orders = t_struct.traders.lock().unwrap();

		// Vector of tuples to construct JSON messages
		let mut to_send: Vec<_> = Vec::new();

		let length_before = orders.len();

		// Iterate through hashmap and filter out orders based on rng
		orders.retain(|_, order| {
			let rand = rng.gen_range(0, upper);
			// order was randomly selected to be cancelled
			if rand == 1 {
				// copy order's params for cancel json
				let mut p = params_for_json(order);
				// update OrderType to be a cancel order
				p.1 = OrderType::Cancel;
				to_send.push(p)
			}

			// (1 / upper) chance of cancelling the given order
			!(rand == 1)
		});

		assert_eq!(length_before, orders.len() + to_send.len());
		to_send
	}

/// Generates a random number of Bid and Ask orders all of OrderType::Enter
/// and returns them in a vector.
pub fn rand_enters(upper: u64) -> Vec<Order> {
	let mut rng = thread_rng();
	let mut orders = Vec::<Order>::new();

	for _ in 0..rng.gen_range(0, upper) {
		orders.push(rand_bid_enter());
	}

	for _ in 0..rng.gen_range(0, upper) {
		orders.push(rand_ask_enter());
	}
	orders
}

/// Generates a random Ask order of OrderType::Enter
pub fn rand_ask_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	let u_max = gen_u_max();
	Order::new(
		gen_order_id(),
		OrderType::Enter,
		TradeType::Ask,
		p_l,
		p_h,
		u_max,
		p_wise_sup(p_l, p_h, u_max)
	)
}

/// Generates a random Bid order of OrderType::Enter
pub fn rand_bid_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	let u_max = gen_u_max();
	Order::new(
		gen_order_id(),
		OrderType::Enter,
		TradeType::Bid,
		p_l,
		p_h,
		u_max,
		p_wise_dem(p_l, p_h, u_max),
	)
}

/// Randomizes the fields of an order but retains trade_id and trade_type
pub fn rand_update_order(old: &Order) -> Order {
	
    let mut new = match old.trade_type {
    	TradeType::Bid => rand_bid_enter(),
    	TradeType::Ask => rand_ask_enter(),
    };
    new.order_type = OrderType::Update;
    new.trader_id = old.trader_id.clone();
    new
}

/// Create a random pair of prices in increasing order 
pub fn gen_prices() -> (f64, f64) {
	let mut rng = thread_rng();
	let p_l: f64 = rng.gen_range(0.0, 100.0);
	let mut p_h: f64 = rng.gen_range(1.0, 200.0);
	while p_h < p_l {
		p_h = rng.gen_range(1.0, 200.0);
	}
	(p_l, p_h)
}

/// Generate a random trader id from random ascii chars
pub fn gen_order_id() -> String {
	let mut rng = thread_rng();
	let id: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(10)
        .collect();
    id
}

/// Create a random u_max value 
pub fn gen_u_max() -> f64 {
	let mut rng = thread_rng();
	rng.gen_range(0.0, 500.0)
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_new_traders() {
		let t_struct = Traders::new();
		let map = t_struct.traders.lock().unwrap();
		assert_eq!(map.len(), 0);
	}

	#[test]
	fn test_insert_traders() {
		let mut t_struct = Traders::new();
		t_struct.new_trader(rand_bid_enter());
		t_struct.new_trader(rand_ask_enter());

		assert_eq!(t_struct.traders.lock().unwrap().len(), 2);
	}
}














