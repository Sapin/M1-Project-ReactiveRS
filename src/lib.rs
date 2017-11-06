
pub mod runtime;

#[cfg(test)]
mod tests {

	//use std::boxed::Box;
	
	use runtime::{Runtime};

    #[test]
    fn it_works () {
		let mut rt = Runtime::new ();
		rt.on_current_instant (Box::new (|rt : &mut Runtime, ()| -> () {
			rt.on_next_instant (Box::new (|rt : &mut Runtime, ()| -> () {
				rt.on_next_instant (Box::new (|rt : &mut Runtime, ()| -> () {
					println!("42");
				}))
			}))
		}));
		while rt.instant () {
			println!("instant");
		}
    }
}

