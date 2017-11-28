
use std::sync::{Arc,Mutex};
use std::cell::{RefCell};
use std::option::{Option};
use std::collections::{VecDeque};

use runtime::{Runtime,Continuation};
use arrow::{Arrow};
use signal::{Signal};

//  ____                 ____  _                   _ 
// |  _ \ _   _ _ __ ___/ ___|(_) __ _ _ __   __ _| |
// | |_) | | | | '__/ _ \___ \| |/ _` | '_ \ / _` | |
// |  __/| |_| | | |  __/___) | | (_| | | | | (_| | |
// |_|    \__,_|_|  \___|____/|_|\__, |_| |_|\__,_|_|
//                               |___/               

struct PureSignalRuntime {
    emitted : bool,
    waiters : VecDeque<Box<Continuation<()>>>,
    present : VecDeque<(Box<Continuation<()>>,Box<Continuation<()>>)>,
    awaken  : bool,
}

#[derive(Clone)]
pub struct PureSignal {
    rt: Arc<Mutex<RefCell<PureSignalRuntime>>>,
}

pub struct EmitPureSignal (PureSignal);

impl PureSignal {
    
    pub fn new () -> PureSignal {
        PureSignal {rt: Arc::new (Mutex::new (RefCell::new (
            PureSignalRuntime {
                emitted: false,
                waiters: VecDeque::new (),
                present: VecDeque::new (),
                awaken : false,
            },
        )))}
    }

    pub fn emit (&self) -> EmitPureSignal {
        EmitPureSignal (self.clone ())
    }

    fn awake (&self, rt: &mut Runtime, data: &mut PureSignalRuntime) {
        if data.awaken {} else {
            data.awaken = true;
            let signal = self.clone ();
            rt.on_end_of_instant (Box::new (move |rt: &mut Runtime, ()| {
                let data = signal.rt.lock ().unwrap ();
                let mut data = data.borrow_mut ();
                (*data).emitted = false;
                (*data).awaken  = false;
                while let Option::Some ((_,ct)) = (*data).present.pop_front () {
                    rt.on_current_instant (ct);
                }
            }));
        }
    }

}

impl Signal for PureSignal {

    fn call_await_immediate (&self, rt: &mut Runtime,
                             next: Box<Continuation<()>>)
    {
        let data = self.rt.lock ().unwrap ();
        let mut data = data.borrow_mut ();
        if (*data).emitted {
            rt.on_current_instant (next);
        } else {
            (*data).waiters.push_back (next);
        }
    }

    fn call_present (&self, rt: &mut Runtime,
                     ifp: Box<Continuation<()>>,
                     ifn: Box<Continuation<()>>)
    {
        let data = self.rt.lock ().unwrap ();
        let mut data = data.borrow_mut ();
        if (*data).emitted {
            rt.on_current_instant (ifp);
        } else {
            (*data).present.push_back ((ifp,ifn));
            self.awake (rt, &mut data);
        }
    }

}

impl Arrow<(),()> for EmitPureSignal {

    fn call<F> (&self, rt: &mut Runtime, (): (), next: F)
    where F: Continuation<()> {
        let &EmitPureSignal(ref signal) = self;
        let data = signal.rt.lock ().unwrap ();
        let mut data = data.borrow_mut ();
        if (*data).emitted {} else {
            (*data).emitted = true;
            while let Option::Some (ct) = (*data).waiters.pop_front () {
                rt.on_current_instant (ct);
            };
            while let Option::Some ((ct,_)) = (*data).present.pop_front () {
                rt.on_current_instant (ct);
            }
            signal.awake (rt, &mut data);
        };
        rt.on_current_instant (Box::new (next));
    }

}
