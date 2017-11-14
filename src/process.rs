
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

pub enum LoopStatus<V> { Continue, Exit(V) }

/// A process that can be executed multiple times, modifying its environment each time
pub trait ProcessMut: Process {
    /// Executes the mutable process in the runtime, then calls `next` with the process and the
    /// process's return value.
    fn call_mut<C>(self, runtime: &mut Runtime, next: C) where
        Self: Sized, C: Continuation<(Self, Self::Value)>;

    /// Executes the process while it returns Continue, call the continuation with v on Exit(v)
    fn loop_while(self) -> While <Self>
    where Self: Sized {
        While {process: self}
    }

    /// Execute the process as a mutable process
    fn execute_mut (self) -> Self::Value
    where Self: Sized {
        let mut rt = Runtime::new ();
        let val = Arc::new (RefCell::new (Option::None));
        let back = val.clone ();
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            self.call_mut (rt,move |_:&mut Runtime, (_,v) : (Self,Self::Value)| {
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

impl<V> ProcessMut for Constant<V>
where V: 'static + Clone
{
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        next.call (runtime, (Constant {value: self.value.clone()}, self.value))
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

impl<P> ProcessMut for Paused<P>
where P: ProcessMut
{
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        self.process.call_mut (runtime, next.map(|(psd,v) : (P,Self::Value)| (psd.pause(),v)).pause())
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

impl<P,F,V> ProcessMut for Map<P,F>
where P: ProcessMut, F: FnMut(P::Value) -> V + 'static
{
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        let mut f = self.map;
        self.process.call_mut (runtime, next.map(move |(p,v) : (P,P::Value)| {
            let r = f(v);
            (p.map(f), r)
        }))
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

impl<P> ProcessMut for Flatten<P>
where P: ProcessMut, P::Value: Process
{
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        self.process.call_mut (runtime,
            |rt: &mut Runtime, (parent, process): (P, P::Value)| {
                process.call (rt, next.map(|v : <P::Value as Process>::Value| {
                    (parent.flatten(), v)
                }))
            })
    }
}

//      _       _       
//     | | ___ (_)_ __  
//  _  | |/ _ \| | '_ \ 
// | |_| | (_) | | | | |
//  \___/ \___/|_|_| |_|
//  

pub struct JoinPoint<A,B,C> {
    a    : Option <A>,
    b    : Option <B>,
    cont : Option <C>
}

impl<A,B,C> JoinPoint<A,B,C> 
where C: Continuation<(A,B)> {
    fn seta(self: &mut Self, rt: &mut Runtime, a : A) {
        let mut vb = Option::None;
        swap (&mut (*self).b, &mut vb);
        match vb {
            Option::None => {
                let mut va = Option::Some (a);
                swap (&mut (*self).a, &mut va);
            }
            Option::Some (b) => {
                let mut next = Option::None;
                swap (&mut (*self).cont, &mut next);
                match next {
                    Option::None => { panic!(); }
                    Option::Some (next) => {
                        next.call (rt, (a,b));
                    }
                }
            }
        }
    }

    fn setb(self: &mut Self, rt: &mut Runtime, b : B) {
        let mut va = Option::None;
        swap (&mut (*self).a, &mut va);
        match va {
            Option::None => {
                let mut vb = Option::Some (b);
                swap (&mut (*self).b, &mut vb);
            }
            Option::Some (a) => {
                let mut next = Option::None;
                swap (&mut (*self).cont, &mut next);
                match next {
                    Option::None => { panic!(); }
                    Option::Some (next) => {
                        next.call (rt, (a,b));
                    }
                }
            }
        }
    }
}

pub struct Join<P,Q> {
    p : P,
    q : Q
}

pub fn join<P,Q> (p: P, q: Q) -> Join<P,Q>
where P: Process, Q: Process {
    Join {p: p, q: q}
}

impl<P,Q> Process for Join<P,Q>
where P: Process, Q: Process
{
    type Value = (P::Value, Q::Value);

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        let join_point = Arc::new (RefCell::new (
            JoinPoint {a: Option::None, b: Option::None, cont: Option::Some(next)}
        ));

        let ja = join_point.clone ();
        let p = self.p;
        runtime.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            p.call (rt, move |rt: &mut Runtime, a: P::Value| {
                let mut j = (*ja).borrow_mut ();
                (*j).seta(rt, a)
            });
        }));

        let jb = join_point.clone ();
        let q = self.q;
        runtime.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            q.call (rt, move |rt: &mut Runtime, b: Q::Value| {
                let mut j = (*jb).borrow_mut ();
                (*j).setb(rt, b)
            });
        }));
    }
}

impl<P,Q> ProcessMut for Join<P,Q>
where P: ProcessMut, Q: ProcessMut
{
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        let join_point = Arc::new (RefCell::new (
            JoinPoint {a: Option::None, b: Option::None,
                cont: Option::Some(next.map (|((p,pv),(q,qv)) : ((P,P::Value),(Q,Q::Value))| {
                    (join(p,q), (pv, qv))
                }))
            }
        ));

        let ja = join_point.clone ();
        let p = self.p;
        runtime.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            p.call_mut (rt, move |rt: &mut Runtime, a: (P, P::Value)| {
                let mut j = (*ja).borrow_mut ();
                (*j).seta(rt, a)
            });
        }));

        let jb = join_point.clone ();
        let q = self.q;
        runtime.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            q.call_mut (rt, move |rt: &mut Runtime, b: (Q, Q::Value)| {
                let mut j = (*jb).borrow_mut ();
                (*j).setb(rt, b)
            });
        }));
    }
}

// __        ___     _ _
// \ \      / / |__ (_) | ___
//  \ \ /\ / /| '_ \| | |/ _ \
//   \ V  V / | | | | | |  __/
//    \_/\_/  |_| |_|_|_|\___|
// 

pub struct While<P> {
    process: P
}

impl<P,V> Process for While<P>
where P: ProcessMut<Value = LoopStatus<V>> {
    type Value = V;

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        let process = self.process;
        process.call_mut(runtime, |rt: &mut Runtime, (p,v) : (P,LoopStatus<V>)| {
            match v {
                LoopStatus::Continue => p.loop_while().call(rt, next),
                LoopStatus::Exit(v)  => next.call(rt, v)
            }
        })
    }
}

impl<P,V> ProcessMut for While<P>
where P: ProcessMut<Value = LoopStatus<V>> {
    fn call_mut<C>(self, runtime: &mut Runtime, next: C)
    where Self: Sized, C: Continuation<(Self, Self::Value)> {
        let process = self.process;
        process.call_mut(runtime, |rt: &mut Runtime, (p,v) : (P,LoopStatus<V>)| {
            match v {
                LoopStatus::Continue => p.loop_while().call_mut(rt, next),
                LoopStatus::Exit(v)  => next.call(rt, (p.loop_while(), v))
            }
        })
    }
}

