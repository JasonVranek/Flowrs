use core::cmp::{min, max};
use std::sync::Arc;
use crate::exchange::order_book::Book;
use rayon::prelude::*;
use decimal::*;

pub fn test_auction_mod() {
	println!("Hello, auction");
}

// Iterate over each order in parallel and compute
// the closure for each. 
pub fn calc_aggs(p: f64, bids: Arc<Book>, asks: Arc<Book>) -> (f64, f64) {
	let bids = bids.orders.lock().unwrap();
	let asks = asks.orders.lock().unwrap();

	let agg_demand: f64 = bids.par_iter()
	    .map(|order| {
	    	if p <= order.p_low {
	    		order.u_max
	    	} else if p > order.p_high {
	    		0.0
	    	} else {
	    		order.calculate(p)
	    	}
	    }).sum();

	let agg_supply: f64 = asks.par_iter()
	    .map(|order| {
	    	if p < order.p_low {
	    		0.0
	    	} else if p >= order.p_high {
	    		order.u_max
	    	} else {
	    		order.calculate(p)
	    	}
	    }).sum();

	(agg_demand, agg_supply)
}

pub fn bs_cross(bids: Arc<Book>, asks: Arc<Book>) -> f64 {
	// get_price_bounds obtains locks on the book's prices
    let (left, right) = get_price_bounds(Arc::clone(&bids), Arc::clone(&asks));


    1.1
}

pub fn get_price_bounds(bids: Arc<Book>, asks: Arc<Book>) -> (f64, f64) {
	let bids_min: f64 = bids.get_min_price();
	let bids_max: f64 = bids.get_max_price();
	let asks_min: f64 = asks.get_min_price();
	let asks_max: f64 = asks.get_max_price();

	(min_float(&bids_min, &asks_min), max_float(&bids_max, &asks_max))
}

pub fn max_float(a: &f64, b: &f64) -> f64 {
    match a.partial_cmp(b).unwrap() {
		std::cmp::Ordering::Less => *b,
		std::cmp::Ordering::Greater => *a,
		std::cmp::Ordering::Equal => *a
	}
}

pub fn min_float(a: &f64, b: &f64) -> f64 {
    match a.partial_cmp(b).unwrap() {
		std::cmp::Ordering::Less => *a,
		std::cmp::Ordering::Greater => *b,
		std::cmp::Ordering::Equal => *a
	}
}


#[test]
fn test_par_iter() {
	let big_sum: u32 = (0..10).collect::<Vec<u32>>()
		.par_iter()
	    .map(|x| x * x)
	    .sum();

	assert_eq!(big_sum, 285);
}

#[test]
fn test_min_max_float() {
	let a = 2.0;
	let b = 10.0;
	assert_eq!(2.0, min_float(&a, &b));
	assert_eq!(10.0, max_float(&a, &b));
}

#[test]
fn test_decimal() {
	let x: d128 = d128!(1.23);
	let y = d128!(1.2);
	let z = d128!(0.03);

	assert_eq!(x, y + z);

}













