use crate::controller::{Task, State};
use crate::exchange::order_book::Book;

use std::sync::{Mutex, Arc};

use rayon::prelude::*;
use crate::utility::get_time;


const EPSILON: f64 =  0.000_000_001;

pub struct Auction {}

impl Auction {
	// Iterate over each order in parallel and compute
	// the closure for each. 
	pub fn calc_aggs(p: f64, bids: Arc<Book>, asks: Arc<Book>) -> (f64, f64) {
		let bids = bids.orders.lock().expect("ERROR: No bids book");
		let asks = asks.orders.lock().expect("ERROR: No asks book");

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

	/// Calculates the market clearing price from the bids and asks books. Uses a 
	/// binary search to find the intersection point between the aggregates supply and 
	/// demand curves. 
	pub fn bs_cross(bids: Arc<Book>, asks: Arc<Book>) -> Option<f64> {
		// get_price_bounds obtains locks on the book's prices
	    let (mut left, mut right) = Auction::get_price_bounds(Arc::clone(&bids), Arc::clone(&asks));
	    let max_iters = 1000;
	    let mut curr_iter = 0;
	    println!("Min Book price: {}, Max Book price: {}", left, right);
	    while left < right {
	    	curr_iter += 1;
	    	// Find a midpoint with the correct price tick precision
	    	let index: f64 = (left + right) / 2.0;
	    	// Calculate the aggregate supply and demand at this price
	    	let (dem, sup) = Auction::calc_aggs(index, Arc::clone(&bids), Arc::clone(&asks));
	    	// println!("price_index: {}, dem: {}, sup: {}", index, dem, sup);

	    	if Auction::greater_than_e(&dem, &sup) {  		// dev > sup
	    		// We are left of the crossing point
	    		left = index;
	    	} else if Auction::less_than_e(&dem, &sup) {	// sup > dem
	    		// We are right of the crossing point
	    		right = index;
	    	} else {
	    		println!("Found cross at: {}", index);
	    		return Some(index);
	    	}

	    	if curr_iter == max_iters {
	    		println!("Trouble finding cross in max iterations, got: {}", index);
	    		return Some(index);
	    	}
	    }
	    None
	}

	/// Schedules an auction to run on an interval determined by the duration parameter in milliseconds.
	/// Outputs a task that will be dispatched asynchronously via the controller module.
	pub fn async_auction_task(bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>, duration: u64) -> Task {
		Task::rpt_task(move || {
			{
	    		// Obtain lock on the global state and switch to Auction mode, will stop
	    		// the queue from being processed.
	    		let mut state = state.lock().unwrap();
	    		*state = State::Auction;
	    	}
	    	println!("Starting Auction @{:?}", get_time());
	    	if let Some(cross_price) = Auction::bs_cross(Arc::clone(&bids), Arc::clone(&asks)) {
	    		println!("Found Cross at @{:?} \nP = {}\n", get_time(), cross_price);
	    	} else {
	    		println!("Error, Cross not found\n");
	    	}
	    	
	    	{
	    		// Change the state back to process to allow the books to be mutated again
	    		let mut state = state.lock().unwrap();
	    		*state = State::Process;
	    	}
		}, duration)
	}

	pub fn get_price_bounds(bids: Arc<Book>, asks: Arc<Book>) -> (f64, f64) {		
		let bids_min: f64 = bids.get_min_price();
		let bids_max: f64 = bids.get_max_price();
		let asks_min: f64 = asks.get_min_price();
		let asks_max: f64 = asks.get_max_price();

		(Auction::min_float(&bids_min, &asks_min), Auction::max_float(&bids_max, &asks_max))
	}

	fn max_float(a: &f64, b: &f64) -> f64 {
	    match a.partial_cmp(b).unwrap() {
			std::cmp::Ordering::Less => *b,
			std::cmp::Ordering::Greater => *a,
			std::cmp::Ordering::Equal => *a
		}
	}

	fn min_float(a: &f64, b: &f64) -> f64 {
	    match a.partial_cmp(b).unwrap() {
			std::cmp::Ordering::Less => *a,
			std::cmp::Ordering::Greater => *b,
			std::cmp::Ordering::Equal => *a
		}
	}

	// true if a > b
	pub fn greater_than_e(a: &f64, b: &f64) -> bool {
		let a = a.abs();
		let b = b.abs();
	    if (a - b).abs() > EPSILON && a - b > 0.0 {
	    	return true;
	    } else {
	    	return false;
	    }
	}

	// true if a < b
	pub fn less_than_e(a: &f64, b: &f64) -> bool {
		let a = a.abs();
		let b = b.abs();
	    if (a - b).abs() > EPSILON && a - b < 0.0 {
	    	return true;
	    } else {
	    	return false;
	    }
	}

	pub	fn equal_e(a: &f64, b: &f64) -> bool {
	    if (a - b).abs() < EPSILON {
	    	return true;
	    } else {
	    	return false;
	    }
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
	assert_eq!(2.0, Auction::min_float(&a, &b));
	assert_eq!(10.0, Auction::max_float(&a, &b));
}

#[test]
fn test_float_helpers() {
	let a = 2.0;
	let b = 10.0;
	assert_eq!(2.0, Auction::min_float(&a, &b));
	assert_eq!(10.0, Auction::max_float(&a, &b));

	assert!(!Auction::greater_than_e(&a, &b));
	assert!(Auction::less_than_e(&a, &b));
	assert!(Auction::equal_e(&(1.1 + 0.4), &1.5));
}













