use crate::io::order::*;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Mutex;


pub struct Traders {
	traders: Mutex<HashMap<String, Order>>,
}

impl Traders {
	pub fn new() -> Self {
		Traders {
			traders: Mutex::new(HashMap::new()),
		}
	}

	pub fn new_trader(&mut self, order: Order) {
		// or_insert will not overwrite an existing entry, but will insert if the key doesn't exist
		let mut traders = self.traders.lock().unwrap();
		traders.entry(order.trader_id.clone()).or_insert(order);
	}

	pub fn new_traders(&self, orders: Vec<Order>) {
		// or_insert will not overwrite an existing entry, but will insert if the key doesn't exist
		let mut traders = self.traders.lock().unwrap();
		for order in orders {
			traders.entry(order.trader_id.clone()).or_insert(order);
		}
	}

	pub fn update_trader(&mut self, order: Order) {
		self.traders.lock().unwrap().insert(order.trader_id.clone(), order);
	}

	pub fn del_trader(&mut self, trader_id: String) {
		self.traders.lock().unwrap().remove(&trader_id);
	}
}

pub fn rand_enters(upper: u64) -> Vec<Order> {
	let mut rng = rand::thread_rng();
	let mut orders = Vec::<Order>::new();

	for _ in 0..rng.gen_range(0, upper) {
		orders.push(rand_bid_enter());
	}

	for _ in 0..rng.gen_range(0, upper) {
		orders.push(rand_ask_enter());
	}
	orders
}



pub fn rand_ask_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	// let coefs = rand_coef_vector();
	let id = gen_order_id();
	let u_max = gen_u_max(500.0);
	Order::new(
		id,
		OrderType::Enter,
		TradeType::Ask,
		p_l,
		p_h,
		u_max,
		p_wise_sup(p_l, p_h, u_max)
		// poly_clos_from_coef(coefs),,
	)
}

pub fn rand_bid_enter() -> Order {
	let (p_l, p_h) = gen_prices();
	// let coefs = rand_coef_vector();
	let id = gen_order_id();
	let u_max = gen_u_max(500.0);
	Order::new(
		id,
		OrderType::Enter,
		TradeType::Bid,
		p_l,
		p_h,
		u_max,
		p_wise_dem(p_l, p_h, u_max),
		// poly_clos_from_coef(coefs),
	)
}

pub fn rand_update_order(old: Order) -> Order {
	// randomizes the fields of an order but retains trade_id and trade_type
    let mut new = match old.trade_type {
    	TradeType::Bid => rand_bid_enter(),
    	TradeType::Ask => rand_ask_enter(),
    };

    new.trader_id = old.trader_id.clone();
    new

}

pub fn gen_prices() -> (f64, f64) {
	// Create a random pair of (p_low, p_high)
	let mut rng = rand::thread_rng();
	let mut p_l: f64 = rng.gen();
	p_l *= 50.0;
	let mut p_h: f64 = rng.gen();
	p_h *= 100.0;
	while p_h < p_l {
		p_h = rng.gen();
		p_h *= 100.0;
	}
	(p_l, p_h)
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

pub fn gen_u_max(scaler: f64) -> f64 {
	// Create a random scaled u_max value 
	let mut rng = rand::thread_rng();
	let u_max: f64 = rng.gen();
	u_max * scaler
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

	// #[test]
	// fn test_rand_enters() {
	// 	let orders = rand_enters(100);
	// 	assert_eq!(orders.len(), 5);
	// }
}














