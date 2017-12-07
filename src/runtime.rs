
use std::collections::VecDeque;
use std::mem::{swap};

//   ____            _   _                   _   _             
//  / ___|___  _ __ | |_(_)_ __  _   _  __ _| |_(_) ___  _ __  
// | |   / _ \| '_ \| __| | '_ \| | | |/ _` | __| |/ _ \| '_ \ 
// | |__| (_) | | | | |_| | | | | |_| | (_| | |_| | (_) | | | |
//  \____\___/|_| |_|\__|_|_| |_|\__,_|\__,_|\__|_|\___/|_| |_|
//                                                             

pub trait Continuation<V> {
    fn call (self, runtime: &mut Runtime, val: V);
    fn call_box (self: Box<Self>, runtime: &mut Runtime, val: V);
}

impl<V,F> Continuation<V> for F
where F: FnOnce(&mut Runtime, V) {
    fn call (self, runtime: &mut Runtime, val: V) {
        self (runtime, val);
    }

    fn call_box (self: Box<Self>, runtime: &mut Runtime, val: V) {
        (*self).call (runtime, val);
    }
}

//  ____              _   _                
// |  _ \ _   _ _ __ | |_(_)_ __ ___   ___ 
// | |_) | | | | '_ \| __| | '_ ` _ \ / _ \
// |  _ <| |_| | | | | |_| | | | | | |  __/
// |_| \_\\__,_|_| |_|\__|_|_| |_| |_|\___|
//                                         

pub struct Runtime {
	current_instant : VecDeque <Box<Continuation<()>>>,
	endof_instant   : VecDeque <Box<Continuation<()>>>,
	next_instant    : VecDeque <Box<Continuation<()>>>,
}

impl Runtime {
    pub fn new () -> Self { Runtime {
        current_instant : VecDeque::new (),
        endof_instant   : VecDeque::new (),
        next_instant    : VecDeque::new (),
    }}

    pub fn execute (&mut self) {
        while self.instant () {}
    }

    pub fn instant (&mut self) -> bool {
        while let Some (ct) = self.current_instant.pop_front () {
            Continuation::call_box (ct, self, ());
        };
        swap (&mut self.current_instant, &mut self.next_instant);
        while let Some (ct) = self.endof_instant.pop_front () {
            Continuation::call_box (ct, self, ());
        };
        ! (self.current_instant.is_empty ())
    }

	pub fn on_current_instant (&mut self, c: Box<Continuation<()>>) {
		self.current_instant.push_back (c)
	}

	pub fn on_next_instant (&mut self, c: Box<Continuation<()>>) {
		self.next_instant.push_back (c)
	}

	pub fn on_end_of_instant (&mut self, c: Box<Continuation<()>>) {
		self.endof_instant.push_back (c)
	}
}

