use crate::order::{Order, OrderType, TradeType};
use crate::exchange::queue::Queue;
use crate::exchange::order_book::Book;
use crate::controller::{Task, State};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Mutex, Arc};

pub struct QueueProcessor {}

impl QueueProcessor {
	// Concurrently process orders in the queue. Each order is
	// either of OrderType::{Enter, Update, Cancel}. Each order will
	// modify the state of either the Bids or Asks Book, but must
	// first acquire a lock on the respective book. 
	pub fn conc_process_order_queue(queue: Arc<Queue>, 
									bids: Arc<Book>, 
									asks: Arc<Book>) 
									-> Vec<JoinHandle<()>>{
		// Acquire lock of Queue
		// Pop off contents of Queue
		// match over the OrderType
		// process each order based on OrderType
		let mut handles = Vec::<JoinHandle<()>>::new();
		for order in queue.pop_all() {
			let handle = match order.trade_type {
				TradeType::Bid => {
					match order.order_type {
						OrderType::Enter => QueueProcessor::process_enter(order, Arc::clone(&bids)),
						OrderType::Update => QueueProcessor::process_update(order, Arc::clone(&bids)),
	  	    			OrderType::Cancel => QueueProcessor::process_cancel(order, Arc::clone(&bids)),
					}
				}
				TradeType::Ask => {
					match order.order_type {
						OrderType::Enter => QueueProcessor::process_enter(order, Arc::clone(&asks)),
						OrderType::Update => QueueProcessor::process_update(order, Arc::clone(&asks)),
	  	    			OrderType::Cancel => QueueProcessor::process_cancel(order, Arc::clone(&asks)),
					}
				}
			};
			handles.push(handle);
		}
		handles
	}


	// Adds the order to the Bids or Asks Book
	fn process_enter(order: Order, book: Arc<Book>) -> JoinHandle<()> {
		// Spawn a new thread to process the order
	    thread::spawn(move || {
	    	// add_order acquires the lock on the book before mutating
	    	book.update_max_price(&order.p_high);
	    	book.update_min_price(&order.p_low);
	    	match book.add_order(order) {
	    		Ok(()) => {},
	    		Err(e) => {
	    			println!("ERROR: {}", e);
	    			// TODO send an error response over TCP
	    		},
	    	}
	    })
	}

	// Updates an order in the Bids or Asks Book in it's own thread
	fn process_update(order: Order, book: Arc<Book>) -> JoinHandle<()> {
		// update books min/max price if this overwrites current min/max OR this order contains new min/max
	    thread::spawn(move || {
	    	let p_high = order.p_high.clone();
			let p_low = order.p_low.clone();
	    	// If the order is not found, bubble error up
	    	match book.update_order(order) {
	    		Ok(()) => {},
	    		Err(e) => {
	    			println!("ERROR: {}", e);
	    			// TODO send an error response over TCP
	    		}
	    	}

	    	let max_p = book.get_max_price();
	    	let min_p = book.get_min_price();

			if p_high == max_p {
	    		// The order previously had the max market price
				book.find_new_max();
			} else if p_high > max_p {
				// The order has a new max market price
				book.update_max_price(&p_high);
			}
			
			if p_low == min_p && p_low != 0.0 {
				// The order previously had the min market price
				book.find_new_min();
				println!("Cancelling old min price");
			} else if p_low < min_p {
				// The order has a new min market price
				book.update_min_price(&p_low);
			}

	    })
	}

	// Cancels the order living in the Bids or Asks Book
	fn process_cancel(order: Order, book: Arc<Book>) -> JoinHandle<()> {
	    thread::spawn(move || {
			let p_high = order.p_high.clone();
			let p_low = order.p_low.clone();

			// If the cancel fails bubble error up.
			match book.cancel_order(order) {
	    		Ok(()) => {},
	    		Err(e) => {
	    			println!("ERROR: {}", e);
	    			// TODO send an error response over TCP
	    		}
	    	}

			// update min/max if we just cancelled previous min/max
			if p_high == book.get_max_price() {
				book.find_new_max();
			}
			if p_low == book.get_min_price() && p_low != 0.0 {
				book.find_new_min();
				println!("Cancelling old min price");
			}
	    	
	    })
	}

	pub fn async_queue_task(queue: Arc<Queue>, 
							bids: Arc<Book>, 
							asks: Arc<Book>, 
							state: Arc<Mutex<State>>, 
							duration: u64) -> Task
	{
	    Task::rpt_task(move || {
	    	match *state.lock().expect("Couldn't lock state in queue task") {
				State::Process => {
					let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
								Arc::clone(&bids),
								Arc::clone(&asks));

					for h in handles {
						h.join().expect("Couldn't join queue tasks");
					}
					// println!("Processing order queue");
				},
				State::Auction => println!("Can't process order queue because auction!"),
				State::PreAuction => println!("Can't process order queue because pre-auction!"),
			}
	    }, duration)
	}
}
