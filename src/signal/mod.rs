
use std::sync::{Arc,Mutex};
use std::cell::{RefCell};
use std::option::{Option};
use std::mem::{swap};

use runtime::{Runtime,Continuation};
use arrow::{Arrow};

pub mod prim;

//  ____  _                   _ 
// / ___|(_) __ _ _ __   __ _| |
// \___ \| |/ _` | '_ \ / _` | |
//  ___) | | (_| | | | | (_| | |
// |____/|_|\__, |_| |_|\__,_|_|
//          |___/               

pub trait Signal : Sized + Clone + Send + 'static {

    fn call_await_immediate (&self, rt: &mut Runtime,
                             next: Box<Continuation<()> + Send>);

    fn call_present (&self, rt: &mut Runtime,
                     ifp: Box<Continuation<()> + Send>,
                     ifn: Box<Continuation<()> + Send>);

    fn await_immediate (&self) -> AwaitImmediate<Self> {
        AwaitImmediate {signal: self.clone ()}
    }

    fn present<A,B,X,Y> (&self, ifp: X, ifn: Y) -> Present<Self,X,Y>
    where A: Send + 'static,
          B: Send + 'static,
          X: Arrow<A,B>,
          Y: Arrow<A,B>,
    {
        Present {
            signal: self.clone (),
            ifp: Arc::new (ifp),
            ifn: Arc::new (ifn),
        }
    }

}

//     _                _ _   ___                              _ _       _       
//    / \__      ____ _(_) |_|_ _|_ __ ___  _ __ ___   ___  __| (_) __ _| |_ ___ 
//   / _ \ \ /\ / / _` | | __|| || '_ ` _ \| '_ ` _ \ / _ \/ _` | |/ _` | __/ _ \
//  / ___ \ V  V / (_| | | |_ | || | | | | | | | | | |  __/ (_| | | (_| | ||  __/
// /_/   \_\_/\_/ \__,_|_|\__|___|_| |_| |_|_| |_| |_|\___|\__,_|_|\__,_|\__\___|
//                                                                               

pub struct AwaitImmediate<S> {
    signal: S,
}

impl<A,S> Arrow<A,A> for AwaitImmediate<S>
where A: Send + 'static,
      S: Signal,
{

    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<A> + Send {
        self.signal.call_await_immediate (rt, Box::new (|rt: &mut Runtime, ()| {
            next.call (rt, a);
        }));
    }

}

//  ____                           _   
// |  _ \ _ __ ___  ___  ___ _ __ | |_ 
// | |_) | '__/ _ \/ __|/ _ \ '_ \| __|
// |  __/| | |  __/\__ \  __/ | | | |_ 
// |_|   |_|  \___||___/\___|_| |_|\__|
//                                     

pub struct Present<S,X,Y> {
    signal: S,
    ifp: Arc<X>,
    ifn: Arc<Y>,
}

impl<A,B,S,X,Y> Arrow<A,B> for Present<S,X,Y>
where A: Send + 'static,
      B: Send + 'static,
      S: Signal,
      X: Arrow<A,B> + Send + Sync,
      Y: Arrow<A,B> + Send + Sync,
{

    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<B> + Send {
        let ifp = self.ifp.clone ();
        let ifn = self.ifn.clone ();
        let val_p = Arc::new (Mutex::new (RefCell::new (Option::Some (a))));
        let val_n = val_p.clone ();
        let next_p = Arc::new (Mutex::new (RefCell::new (Option::Some (next))));
        let next_n = next_p.clone ();
        self.signal.call_present (rt,
            Box::new (move |rt: &mut Runtime, ()| {
                let mut val = Option::None;
                let mut next = Option::None;
                {
                    let mut val_p = val_p.lock ().unwrap ();
                    swap (&mut *(*val_p).borrow_mut (), &mut val );
                }
                {
                    let mut next_p = next_p.lock ().unwrap ();
                    swap (&mut *(*next_p).borrow_mut (), &mut next);
                }
                ifp.call (rt, val.unwrap (), next.unwrap ());
            }),
            Box::new (move |rt: &mut Runtime, ()| {
                let mut val = Option::None;
                let mut next = Option::None;
                {
                    let mut val_n = val_n.lock ().unwrap ();
                    swap (&mut *(*val_n ).borrow_mut (), &mut val );
                }
                {
                    let mut next_n = next_n.lock ().unwrap ();
                    swap (&mut *(*next_n).borrow_mut (), &mut next);
                }
                ifn.call (rt, val.unwrap (), next.unwrap ());
            })
        );
    }

}

