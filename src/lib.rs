
pub mod runtime;

#[cfg(test)]
mod tests {

	//use std::boxed::Box;
	
	use runtime::{Runtime,Continuation};

    #[test]
    fn it_works () {
		let mut rt = Runtime::new ();
		rt.on_current_instant (Box::new (|rt : &mut Runtime, ()| -> () {
			rt.on_next_instant (Box::new (|rt : &mut Runtime, ()| -> () {
				rt.on_next_instant (Box::new (|_ : &mut Runtime, ()| -> () {
					println!("42");
				}))
			}))
		}));
		while rt.instant () {
			println!("instant");
		}
    }

    #[test]
    fn it_works_with_pause () {
        let mut rt = Runtime::new ();
        let ct =
            (|_:&mut Runtime, ()| { println!("42"); })
            .pause ()
            .pause ();
        rt.on_current_instant (Box::new (ct));
        while rt.instant () {
            println!("instant");
        }
    }

}

