use std::sync::{Mutex, Arc};
use rand::Rng;
use tokio::prelude::*;
use tokio::timer::Interval;
use std::time::{Duration, Instant, SystemTime};

use crate::exchange::order_book::*;
use crate::io::order::*;
use crate::exchange::auction;


pub mod io;
pub mod exchange;

pub enum State {
	Process,
	PreAuction,
	Auction,
}


pub fn setup() -> (Arc<Queue>, Arc<Book>, Arc<Book>, Arc<Mutex<State>>) {
	let queue = Arc::new(Queue::new());
	let bids_book = Arc::new(Book::new(TradeType::Bid));
	let asks_book = Arc::new(Book::new(TradeType::Ask));
	(queue, bids_book, asks_book, Arc::new(Mutex::new(State::Process)))
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

pub fn setup_orders() -> (Vec<Order>, Vec<Order>) {
	let mut bids = Vec::<Order>::new();
	let mut asks = Vec::<Order>::new();
	for i in 0..100 {
		bids.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Bid, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_dem(i as f64, 100.0, 500.0),
		));
		asks.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Ask, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_sup(i as f64, 100.0, 500.0),
		));

	}

	(bids, asks)
}

pub fn auction_interval(bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>, duration: u64) 
-> Box<Future<Item = (), Error = ()> + Send> 
{
	let task = Interval::new(Instant::now(),  Duration::from_millis(duration))
	    .for_each(move |_| {
	    	{
	    		// Obtain lock on the global state and switch to Auction mode
	    		let mut state = state.lock().unwrap();
	    		*state = State::Auction;
	    	}
	    	let now = SystemTime::now();
	    	println!("Starting Auction");
	    	let cross_price = auction::bs_cross(Arc::clone(&bids), Arc::clone(&asks)).unwrap();
	    	let done = now.elapsed().unwrap().subsec_nanos();
	    	println!("Found Cross at @{} \nP = {}", done, cross_price);
	    	{
	    		// Change the state back to process to allow the books to be mutated again
	    		let mut state = state.lock().unwrap();
	    		*state = State::Process;
	    	}
	    	Ok(())
	    })
	    .map_err(|e| panic!("Auction Delay error; err={:?}", e));

	Box::new(task)
}

pub fn process_queue_interval(queue: Arc<Queue>, bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>) 
-> Box<Future<Item = (), Error = ()> + Send> 
{
    let task = Interval::new(Instant::now(), Duration::from_millis(10))
        .for_each(move |_| {
    		// Obtain lock on the global state and only process if in Process state
    		match *state.lock().unwrap() {
    			State::Process => {
    				let handles = conc_process_order_queue(Arc::clone(&queue), 
								Arc::clone(&bids),
								Arc::clone(&asks));

					for h in handles {
						h.join().unwrap();
					}
					println!("Processing order queue");
    			},
    			State::Auction => println!("Can't process order queue because auction!"),
    			State::PreAuction => println!("Can't process order queue because pre-auction!"),
			}
			Ok(())
        })
        .map_err(|e| println!("Error processing order iterval: {}", e));

    Box::new(task)
}