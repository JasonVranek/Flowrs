use std::sync::Arc;
use crate::exchange::order_book::Book;
use rayon::prelude::*;


pub fn test_auction_mod() {
	println!("Hello, auction");
}

// Iterate over each order in parallel and compute
// the closure for each. 
pub fn calc_aggs(p: f64, bids: Arc<Book>, 
    asks: Arc<Book>) -> (f64, f64) {
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

#[test]
fn test_par_iter() {
	let big_sum: u32 = (0..10).collect::<Vec<u32>>()
		.par_iter()
	    .map(|x| x * x)
	    .sum();

	assert_eq!(big_sum, 285);
}