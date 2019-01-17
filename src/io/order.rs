pub fn test_order_mod() {
	println!("Hello, order!");
}

pub fn add_one(num: u32) -> u32 {
	num + 1
}

// Enum for matching over order types
#[derive(Debug, PartialEq)]
pub enum OrderType {
    Enter,
    Update,
    Cancel,
}

impl Clone for OrderType {
	fn clone(&self) -> OrderType { 
		match self {
			OrderType::Enter => OrderType::Enter,
			OrderType::Update => OrderType::Update,
			OrderType::Cancel => OrderType::Cancel,
		}
	}
}


// Enum for matching over bid or ask
#[derive(Debug, PartialEq)]
pub enum TradeType {
    Bid,
    Ask,
}

impl Clone for TradeType {
	fn clone(&self) -> TradeType { 
		match self {
			TradeType::Ask => TradeType::Ask,
			TradeType::Bid => TradeType::Bid,
		}
	}
}

pub struct Order {
	pub trader_id: String,			// address of the trader
	pub order_type: OrderType,	// Enter, Update, Cancel
	pub trade_type: TradeType,  // Bid, Ask
	pub p_low: f64,				// trader's low price
	pub p_high: f64,			// trader's high price
	pub u_max: f64,
	function: Box<
	    		Fn(f64) -> f64 
	    		+ Send 
	    		+ Sync 
	    		+ 'static>,	    // trader's custom closure on heap
}

impl Order {
    pub fn new(
    	t_id: String, 
    	o_t: OrderType, 
    	t_t: TradeType, 
    	pl: f64, ph: f64, u: f64,
    	function: Box<Fn(f64) -> f64 + Send + Sync + 'static>) -> Order {
    	Order {
    		trader_id: t_id,		
			order_type: o_t,	
			trade_type: t_t,  
			p_low: pl,				
			p_high: ph,	
			u_max: u,		
			function,			
    	}
    }

    // method for calling the order's closure
    pub fn calculate(&self, arg: f64) -> f64 {
    	(self.function)(arg)
    }

    pub fn describe(&self) {
    	println!("Trader Id: {:?} \n OrderType: {:?}
    		p_low: {:?}, p_high: {:?}", 
    		self.trader_id, self.order_type,
    		self.p_low, self.p_high);
    }
}

// impl Clone for Order {
// 	fn clone(&self) -> Order { 
// 		Order {
// 			trader_id: self.trader_id.clone(),
// 			order_type: self.order_type.clone(),	
// 			trade_type: self.trade_type.clone(),  
// 			p_low: self.p_low.clone(),				
// 			p_high: self.p_high.clone(),	
// 			u_max: self.u_max.clone(),		
// 			function: Box::new(|x| -> f64),	
// 		} 
// 	} 	
// }

	/// Creates a closure from an array of floats. This closure is the 
	/// equivalent to a polynomial. 
	/// For example: coef = [3, 5, 4, 1] => 3x^3 + 5x^2 + 4x + 1
    // pub fn poly_clos_from_coef(coefs: &'static [f64]) -> 
    pub fn poly_clos_from_coef(coefs: Vec<f64>) -> 
        Box<Fn(f64) -> f64 + Send + Sync + 'static>
    {
        // let x be a generic f64 input that closure will compute on
        let iter = Box::new(move |x: f64| -> f64 {
        	// rev since enumerate counts from 0 up, and we wish
        	// to extract out the index which corresponds to the poly's
        	// degree.
        	coefs.iter().rev().enumerate()
        	    .map(|(deg, coef)| {
        	    	// deg = index in rev order
        	    	// coef = poly's coef from vector
        	    	let eval: f64 = coef * x.powi(deg as i32);
        	    	eval
        	    })
        	    .sum()
        });
        iter
    }

    pub fn p_wise_dem(p_l: f64, p_h: f64, u: f64) -> Box<Fn(f64) -> f64 + Send + Sync + 'static> {
    	let func = Box::new(move |x: f64| -> f64 {
    		if x <= p_l {
	    		u
	    	} else if x > p_h {
	    		0.0
	    	} else {
	    		u * ((p_h - x) / (p_h - p_l))
	    	}
    	});
    	func
    }

    pub fn p_wise_sup(p_l: f64, p_h: f64, u: f64) -> Box<Fn(f64) -> f64 + Send + Sync + 'static> {
    	let func = Box::new(move |x: f64| -> f64 {
    		if x < p_l {
	    		0.0
	    	} else if x >= p_h {
	    		u
	    	} else {
	    		u + ((x - p_h) / (p_h - p_l)) * u
	    	}
    	});
    	func
    }


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_add_one() {
		assert_eq!(2, add_one(1));
	}

	#[test]
	fn test_new_order() {
		let order = Order::new(
			String::from("trader_id"),
			OrderType::Enter,
			TradeType::Bid,
			0.0,
			100.0,
			500.0,
			Box::new(|x| {
				println!("This is my closure");
				x + 1 as f64
			})

		);

		assert_eq!(order.trader_id, "trader_id");
		assert_eq!(order.order_type, OrderType::Enter);
		assert_eq!(order.trade_type, TradeType::Bid);
		assert_eq!(order.p_low, 0.0);
		assert_eq!(order.p_high, 100.0);
		assert_eq!(order.calculate(5.0), 6.0);
	}

	#[test]
	fn test_poly_clos_from_coef() {
		// [3, 5, 4, 1] => 3x^3 + 4x^2 + 5x + 1 
		
		let closure = poly_clos_from_coef(vec![3.0, 4.0, 5.0, 1.0]);
		assert_eq!(51.0, closure(2.0));
		assert_eq!(133.0, closure(3.0));
		assert_eq!(277.0, closure(4.0));

		//x=2: 24 + 16 + 10 + 1 = 51
		//x=3: 81 + 36 + 15 + 1 = 133
		//x=4: 192+ 64 + 20 + 1 = 277

		// -3x + 4
		let closure = poly_clos_from_coef(vec![-3.0, 4.0]);

		let order = Order::new(
			String::from("trader_id"),
			OrderType::Enter,
			TradeType::Bid,
			0.0,
			100.0,
			500.0,
			closure
		);

		assert_eq!(-17.0, order.calculate(7.0));
		assert_eq!(19.0, order.calculate(-5.0));
	}

	#[test]
	fn test_piecewise_demand() {
		let (p_l, p_h, u_max) = (100.0, 200.0, 500.0);
		let closure = p_wise_dem(p_l, p_h, u_max);
		// u * ((p_h - x) / (p_h - p_l))
		// 500 * ((200 - 150 / (200 - 100)) = 250
		assert_eq!(closure(50.0), 500.0);
		assert_eq!(closure(150.0), 250.0);
		assert_eq!(closure(300.0), 0.0);
	}

	#[test]
	fn test_piecewise_supply() {
		let (p_l, p_h, u_max) = (100.0, 200.0, 500.0);
		let closure = p_wise_sup(p_l, p_h, u_max);
		// u * ((p_h - x) / (p_h - p_l))
		// 500 * ((200 - 150 / (200 - 100)) = 250
		assert_eq!(closure(50.0), 0.0);
		assert_eq!(closure(150.0), 250.0);
		assert_eq!(closure(300.0), 500.0);
	}
}

























