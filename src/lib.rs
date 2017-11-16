
pub mod runtime;
pub mod process;
pub mod signal;

#[cfg(test)]
mod tests {

    use std::sync::Arc ;
    use std::cell::RefCell ;
	use runtime::{Runtime,Continuation};
    use process::{Process,ProcessMut,value,join,LoopStatus};

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

    #[test]
    fn it_works_with_processes () {
        value (())
            .pause ()
            .pause ()
            .map (|()| { println!("42"); })
        .execute ();
    }

    #[test]
    fn join_works () {
        let p = value ("foo");
        let q = value ("bar");
        join (p,q)
            .map (|(foo,bar)| {
                println! ("{}", foo);
                println! ("{}", bar);
            })
        .execute ();
    }

    #[test]
    fn while_works() {
        let val = Arc::new (RefCell::new (0));
        let back = val.clone ();
        let f = move |()| {
            let mut val = (*back).borrow_mut();
            println!("WHILE test : {}", val);
            *val += 1;
            if *val > 5 {
                LoopStatus::Exit(())
            } else {
                LoopStatus::Continue
            }
        };
        value (())
            .map (f)
            .loop_while ()
            .execute_mut ();
    }

}

