
use std::sync::Arc ;
use std::cell::RefCell ;
use std::option::Option ;
use std::mem::swap ;

use runtime::{Runtime,Continuation} ;

//  ____                              
// |  _ \ _ __ ___   ___ ___  ___ ___ 
// | |_) | '__/ _ \ / __/ _ \/ __/ __|
// |  __/| | | (_) | (_|  __/\__ \__ \
// |_|   |_|  \___/ \___\___||___/___/
//  

pub trait Process: 'static {
    type Value;
    
    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value>;

    fn execute (self) -> Self::Value
    where Self: Sized {
        let mut rt = Runtime::new ();
        let val = Arc::new (RefCell::new (Option::None));
        let back = val.clone ();
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            self.call (rt,move |_:&mut Runtime, v : Self::Value| {
                let mut val = (*back).borrow_mut ();
                *val = Option::Some (v)
            })
        }));
        rt.execute ();
        let mut default = Option::None;
        swap (&mut *(*val).borrow_mut (), &mut default);
        match default {
            Option::None => panic! (),
            Option::Some (v) => v
        }
    }

    fn pause (self) -> Paused <Self>
    where Self: Sized {
        Paused {process : self}
    }

    fn map<F,V> (self, f: F) -> Map <Self,F>
    where Self: Sized, F: FnOnce(Self::Value) -> V + 'static {
        Map {process : self, map : f}
    }

    fn flatten (self) -> Flatten <Self>
    where Self: Sized {
        Flatten {process : self}
    }

    fn and_then<F,P> (self, f: F) -> Flatten <Map <Self,F>>
    where Self: Sized, F: FnOnce(Self::Value) -> P + 'static, P: Process {
        self.map (f).flatten ()
    }

}

pub fn value <V> (v : V) -> Constant<V>
where V: Sized {
    Constant {value : v}
}

//   ____                _              _   
//  / ___|___  _ __  ___| |_ __ _ _ __ | |_ 
// | |   / _ \| '_ \/ __| __/ _` | '_ \| __|
// | |__| (_) | | | \__ \ || (_| | | | | |_ 
//  \____\___/|_| |_|___/\__\__,_|_| |_|\__|
//  

pub struct Constant<V> {
    value: V
}

impl<V> Process for Constant<V>
where V: 'static
{
    type Value = V;

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        next.call (runtime, self.value)
    }
}

//  ____                          _ 
// |  _ \ __ _ _   _ ___  ___  __| |
// | |_) / _` | | | / __|/ _ \/ _` |
// |  __/ (_| | |_| \__ \  __/ (_| |
// |_|   \__,_|\__,_|___/\___|\__,_|
//  

pub struct Paused<P> {
    process : P
}

impl<P> Process for Paused<P>
where P: Process
{
    type Value = P::Value;

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        self.process.call (runtime, next.pause ())
    }
}


//  __  __             
// |  \/  | __ _ _ __  
// | |\/| |/ _` | '_ \ 
// | |  | | (_| | |_) |
// |_|  |_|\__,_| .__/ 
//              |_|    

pub struct Map<P,F> {
    process : P,
    map : F
}

impl<P,F,V> Process for Map<P,F>
where P: Process, F: FnOnce(P::Value) -> V + 'static
{
    type Value = V;

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<V> {
        self.process.call (runtime, next.map (self.map))
    }
}

//  _____ _       _   _             
// |  ___| | __ _| |_| |_ ___ _ __  
// | |_  | |/ _` | __| __/ _ \ '_ \ 
// |  _| | | (_| | |_| ||  __/ | | |
// |_|   |_|\__,_|\__|\__\___|_| |_|
//    

pub struct Flatten<P> {
    process : P
}

impl<P> Process for Flatten<P>
where P: Process, P::Value: Process
{
    type Value = <P::Value as Process>::Value;

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        self.process.call (runtime,
        |rt: &mut Runtime, process: P::Value| {
            process.call (rt, next)
        })
    }
}

