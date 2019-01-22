use tokio::prelude::*;
use futures::future::{join_all};

pub type AsyncTask = Box<Future<Item = (), Error = ()> + Send>;

// Enum for tracking the state of the exchange
#[derive(Debug)]
pub enum State {
	Process,
	PreAuction,
	Auction,
}

// A wrapper around tokio to dispatch tasks asynchronously
pub struct Controller {}

impl Controller {
	pub fn run(tasks: Vec<AsyncTask>) {
		// Use join/join_all to combine futures into a single future to use in tokio::run
		tokio::run(join_all(tasks).map(|_| ()));
	}


}


pub struct Task {}


