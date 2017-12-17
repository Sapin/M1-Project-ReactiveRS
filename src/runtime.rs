
use std::thread;
use std::sync::{Arc,Mutex};
use std::cell::{RefCell};
use std::mem::{swap};
use std::option::{Option};
use std::collections::VecDeque;

//   ____            _   _                   _   _             
//  / ___|___  _ __ | |_(_)_ __  _   _  __ _| |_(_) ___  _ __  
// | |   / _ \| '_ \| __| | '_ \| | | |/ _` | __| |/ _ \| '_ \ 
// | |__| (_) | | | | |_| | | | | |_| | (_| | |_| | (_) | | | |
//  \____\___/|_| |_|\__|_|_| |_|\__,_|\__,_|\__|_|\___/|_| |_|
//                                                             

pub trait Continuation<V> : 'static {
    fn call (self, runtime: &mut Runtime, val: V);
    fn call_box (self: Box<Self>, runtime: &mut Runtime, val: V);
}

impl<V,F> Continuation<V> for F
where F: FnOnce(&mut Runtime, V) + 'static {
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

pub trait Runtime {

    fn on_current_instant (&mut self, c: Box<Continuation<()> + Send>);
    fn on_next_instant    (&mut self, c: Box<Continuation<()> + Send>);
    fn on_end_of_instant  (&mut self, c: Box<Continuation<()> + Send>);

}

//  ____             ____              _   _                
// / ___|  ___  __ _|  _ \ _   _ _ __ | |_(_)_ __ ___   ___ 
// \___ \ / _ \/ _` | |_) | | | | '_ \| __| | '_ ` _ \ / _ \
//  ___) |  __/ (_| |  _ <| |_| | | | | |_| | | | | | |  __/
// |____/ \___|\__, |_| \_\\__,_|_| |_|\__|_|_| |_| |_|\___|
//                |_|                                       

pub struct SeqRuntime {
	current_instant : VecDeque <Box<Continuation<()> + Send>>,
	endof_instant   : VecDeque <Box<Continuation<()> + Send>>,
	next_instant    : VecDeque <Box<Continuation<()> + Send>>,
}

impl SeqRuntime {

    pub fn new () -> Self { SeqRuntime {
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

}

impl Runtime for SeqRuntime {

	fn on_current_instant (&mut self, c: Box<Continuation<()> + Send>) {
		self.current_instant.push_back (c)
	}

	fn on_next_instant    (&mut self, c: Box<Continuation<()> + Send>) {
		self.next_instant.push_back (c)
	}

	fn on_end_of_instant  (&mut self, c: Box<Continuation<()> + Send>) {
		self.endof_instant.push_back (c)
	}

}

//  ____            ____              _   _                
// |  _ \ __ _ _ __|  _ \ _   _ _ __ | |_(_)_ __ ___   ___ 
// | |_) / _` | '__| |_) | | | | '_ \| __| | '_ ` _ \ / _ \
// |  __/ (_| | |  |  _ <| |_| | | | | |_| | | | | | |  __/
// |_|   \__,_|_|  |_| \_\\__,_|_| |_|\__|_|_| |_| |_|\___|
//                                                         

struct ParRuntimeCommon {
    current_instant : VecDeque <Box<Continuation<()> + Send>>,
    endof_instant   : VecDeque <Box<Continuation<()> + Send>>,
    next_instant    : VecDeque <Box<Continuation<()> + Send>>,
    working         : u32,
    running         : bool,
}

pub struct ParRuntime {
    base : Arc<Mutex<RefCell<ParRuntimeCommon>>>,
    next : Option<Box<Continuation<()> + Send>>,
}

impl ParRuntime {

    pub fn new () -> Self { ParRuntime {
        base : Arc::new (Mutex::new (RefCell::new (ParRuntimeCommon {
            current_instant : VecDeque::new (),
            endof_instant   : VecDeque::new (),
            next_instant    : VecDeque::new (),
            running         : true,
            working         : 0,
        }))),
        next : Option::None,
    }}

    pub fn spawn (&self) {
        let base = self.base.clone ();
        thread::spawn(move || {
            let mut child = ParRuntime {
                base : base,
                next : Option::None,
            };
            {
                let base = child.base.lock ().unwrap ();
                let mut base = base.borrow_mut ();
                base.working = base.working + 1;
            }
            loop {
                {
                    let base = child.base.lock ().unwrap ();
                    let mut base = base.borrow_mut ();
                    child.next = base.current_instant.pop_front ();
                    if child.next.is_none () {
                        base.working = base.working - 1;
                    }
                }
                while child.next.is_none () {
                    thread::yield_now ();
                    let base = child.base.lock ().unwrap ();
                    let mut base = base.borrow_mut ();
                    if !base.running { return; }
                    child.next = base.current_instant.pop_front ();
                    if !child.next.is_none () {
                        base.working = base.working + 1;
                    }
                }
                while let Some (ct) = child.next.take () {
                    Continuation::call_box (ct, &mut child, ());
                }
            }
        });
    }

    pub fn execute (&mut self) {
        loop {
            while self.next.is_none () {
                let base = self.base.lock ().unwrap ();
                let mut base = base.borrow_mut ();
                self.next = base.current_instant.pop_front ();
                if self.next.is_none () {
                    if base.working == 0 {
                        swap (&mut base.current_instant, &mut base.endof_instant);
                        base.current_instant.append (&mut base.next_instant);
                        base.running = !base.current_instant.is_empty ();
                        if !base.running { return; }
                        self.next = base.current_instant.pop_front ();
                    } else {
                        thread::yield_now ();
                    }
                }
            }
            while let Some (ct) = self.next.take () {
                Continuation::call_box (ct, &mut self, ());
            }
        }
    }

}

impl Runtime for ParRuntime {

    fn on_current_instant (&mut self, c: Box<Continuation<()> + Send>) {
        if self.next.is_none () {
            self.next = Option::Some (c);
        } else {
            let base = self.base.lock ().unwrap ();
            let mut base = base.borrow_mut ();
            base.current_instant.push_back (c);
        }
    }

    fn on_next_instant    (&mut self, c: Box<Continuation<()> + Send>) {
        let base = self.base.lock ().unwrap ();
        let mut base = base.borrow_mut ();
        base.next_instant.push_back (c);
    }

    fn on_end_of_instant  (&mut self, c: Box<Continuation<()> + Send>) {
        let base = self.base.lock ().unwrap ();
        let mut base = base.borrow_mut ();
        base.endof_instant.push_back (c);
    }

}

