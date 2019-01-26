use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::{Interval, Delay};
use futures::future::join_all;
use futures::{Future};

use futures::future;

#[derive(Debug)]
pub enum State {
	Process,
	PreAuction,
	Auction,
}

// A wrapper around tokio to dispatch tasks asynchronously
pub struct Controller {
	tasks: Vec<AsyncTask>,
}

impl Controller {
	pub fn new() -> Controller {
		Controller{
			tasks: Vec::<AsyncTask>::new(),
		}
	}

	// Pushes contents of Task as an AsyncTask to be run
	pub fn push(&mut self, task: Task) {
		self.tasks.push(task.task);
	}

	pub fn run(self) {
		// Use join/join_all to combine futures into a single future to use in tokio::run
		tokio::run(join_all(self.tasks).map(|_| ()));
	}
}

pub type AsyncTask = Box<Future<Item = (), Error = ()> + Send>;

// A wrapper to easily create dispatch closure's asynchronously as tasks in tokio
pub struct Task {
	pub task: AsyncTask,
}

impl Task {
	// Takes in a closure and returns a Task to run with Tokio
	pub fn new<F>(f: F) -> Task
	where F: Fn() + Send + Sync + 'static, 
	{
		Task {
			task: Box::new(future::lazy(move || {
				f();
				future::ok(())
			}))
		}
	} 

	/// Calls the closure after a specified time in millis
	pub fn delay_task<F>(f: F, millis: u64) -> Task 
	where F: Fn() + Send + Sync + 'static 
	{
		let when = Instant::now() + Duration::from_millis(millis);
		let new_task = Delay::new(when)
		    .and_then(move |_| {
		    	f();
		    	Ok(())
		    })
		    .map_err(|_| ());

		Task{
			task: Box::new(new_task)
		}
	}

	/// Calls the closure on an interval specified by millis 
	pub fn rpt_task<F>(f: F, millis: u64) -> Task 
	where F: Fn() + Send + Sync + 'static 
	{
		let new_task = Interval::new_interval(Duration::from_millis(millis))
		    .for_each(move |_| {
		    	f();
		    	Ok(())
		    })
		    .map_err(|_| ());

		Task{
			task: Box::new(new_task)
		}
	}

	/// Converts a one off task into a delayed task
	pub fn after_delay(self, millis: u64) -> Task {
		let when = Instant::now() + Duration::from_millis(millis);
		let new_task = Delay::new(when)
		    .and_then(|_| {
		    	tokio::spawn(self.task);
		    	Ok(())
		    })
		    .map_err(|_| ());

		Task{
			task: Box::new(new_task)
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::{Arc, Mutex};
	use std::thread;

	#[test]
	fn test_new_task() {
		let number = Arc::new(Mutex::new(10));

		let num1 = Arc::clone(&number);
		let num2 = Arc::clone(&number);
		let num3 = Arc::clone(&number);

		let task1 = Task::new(move || {
			let mut num = num1.lock().unwrap();
			*num += 1;
			println!("Ran task");

		});

		let task2 = Task::delay_task(move || {
			let mut num = num2.lock().unwrap();
			*num += 1;
			println!("Mutated state after delay");

		}, 1000);

		let task3 = Task::delay_task(move || {
			let num3 = num3.clone();
			thread::spawn(move || {
				let mut num = num3.lock().unwrap();
				*num += 1;
				println!("Mutated state after delay in different thread");
			});
		}, 500);

		let mut controller = Controller::new();
		controller.push(task1);
		controller.push(task2);
		controller.push(task3);

		controller.run();

		assert_eq!(*number.lock().unwrap(), 13);
	}

	
}





















