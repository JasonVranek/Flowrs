pub fn test_order_mod() {
	println!("Hello, order!");
}

pub fn add_one(num: u32) -> u32 {
	num + 1
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum OrderType {
    Enter,
    Update,
    Cancel,
}


#[derive(PartialEq)]
#[derive(Debug)]
pub enum TradeType {
    Bid,
    Ask,
}

pub struct Order {
	trader_id: String,				// address of the trader
	order_type: OrderType,			// Enter, Update, Cancel
	trade_type: TradeType,  		// Bid, Ask
	p_low: u32,						// trader's low price
	p_high: u32,					// trader's high price
	function: Box<Fn(f64) -> f64>,	// trader's custom closure
}

impl Order {
    fn new(t_id: String, o_t: OrderType, t_t: TradeType, 
    	    pl: u32, ph: u32, function: Box<Fn(f64) -> f64>) -> Order {
    	Order {
    		trader_id: t_id,		
			order_type: o_t,	
			trade_type: t_t,  
			p_low: pl,				
			p_high: ph,			
			function,			
    	}
    }

    // method for calling the order's closure
    fn calculate(&mut self, arg: f64) -> f64 {
    	(self.function)(arg)
    }
}

	/// Creates a closure from an array of floats. This closure is the 
	/// equivalent to a polynomial. 
	/// For example: coef = [3, 5, 4, 1] => 3x^3 + 5x^2 + 4x + 1
    fn poly_clos_from_coef(coefs: &'static [f64]) -> Box<Fn(f64) -> f64> {
    	
    	let coefs = coefs.clone();

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


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_add_one() {
		assert_eq!(2, add_one(1));
	}

	#[test]
	fn test_new_order() {
		let mut order = Order::new(
			String::from("trader_id"),
			OrderType::Enter,
			TradeType::Bid,
			0,
			100,
			Box::new(|x| {
				println!("This is my closure");
				x + 1 as f64
			})

		);

		assert_eq!(order.trader_id, "trader_id");
		assert_eq!(order.order_type, OrderType::Enter);
		assert_eq!(order.trade_type, TradeType::Bid);
		assert_eq!(order.p_low, 0);
		assert_eq!(order.p_high, 100);
		assert_eq!(order.calculate(5.0), 6.0);
	}

	#[test]
	fn test_poly_clos_from_coef() {
		// [3, 5, 4, 1] => 3x^3 + 4x^2 + 5x + 1 
		let coefs: &'static[f64] = &[3.0, 4.0, 5.0, 1.0];
		
		let closure = poly_clos_from_coef(coefs);
		assert_eq!(51.0, closure(2.0));
		assert_eq!(133.0, closure(3.0));
		assert_eq!(277.0, closure(4.0));

		//x=2: 24 + 16 + 10 + 1 = 51
		//x=3: 81 + 36 + 15 + 1 = 133
		//x=4: 192+ 64 + 20 + 1 = 277

		// -3x + 4
		let closure = poly_clos_from_coef(&[-3.0, 4.0]);

		let mut order = Order::new(
			String::from("trader_id"),
			OrderType::Enter,
			TradeType::Bid,
			0,
			100,
			closure
		);

		assert_eq!(-17.0, order.calculate(7.0));
		assert_eq!(19.0, order.calculate(-5.0));
	}
}

























