
use std::option::{Option};
use std::result::{Result};
use std::clone::{Clone};
use std::cell::{RefCell};
use std::sync::{Arc,Mutex};
use std::mem::{swap};
use std::marker::{PhantomData};

use runtime::{Runtime,Continuation};
use arrow::{Arrow};

//  ___    _            _   _ _         
// |_ _|__| | ___ _ __ | |_(_) |_ _   _ 
//  | |/ _` |/ _ \ '_ \| __| | __| | | |
//  | | (_| |  __/ | | | |_| | |_| |_| |
// |___\__,_|\___|_| |_|\__|_|\__|\__, |
//                                |___/ 

pub struct Identity ();

pub fn identity () -> Identity {
    Identity ()
}

impl<A> Arrow<A,A> for Identity
where A: 'static
{

    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<A> {
        next.call (rt, a);
    }

}

// __     __    _            
// \ \   / /_ _| |_   _  ___ 
//  \ \ / / _` | | | | |/ _ \
//   \ V / (_| | | |_| |  __/
//    \_/ \__,_|_|\__,_|\___|
//                           

pub struct Value<B> {
    val: B,
}

pub fn value<B> (val: B) -> Value<B>
where B: Clone + 'static
{
    Value {val: val}
}

impl<A,B> Arrow<A,B> for Value<B>
where A: 'static,
      B: Clone + 'static
{
    
    fn call<F> (&self, rt: &mut Runtime, _: A, next: F)
    where F: Continuation<B> {
        next.call (rt, self.val.clone ());
    }

}

//  __  __             
// |  \/  | __ _ _ __  
// | |\/| |/ _` | '_ \ 
// | |  | | (_| | |_) |
// |_|  |_|\__,_| .__/ 
//              |_|    

pub struct Map<F> {
    f: F,
}

pub fn map<A,B,F> (f: F) -> Map<F>
where F: Fn(A) -> B + 'static
{
    Map {f: f}
}

impl<A,B,F> Arrow<A,B> for Map<F>
where A: 'static,
      B: 'static,
      F: Fn(A) -> B + 'static
{
    
    fn call<C> (&self, rt: &mut Runtime, a: A, next: C)
    where C: Continuation<B> {
        next.call (rt, (self.f) (a));
    }

}

//  ____                      
// |  _ \ __ _ _   _ ___  ___ 
// | |_) / _` | | | / __|/ _ \
// |  __/ (_| | |_| \__ \  __/
// |_|   \__,_|\__,_|___/\___|
//                            

pub struct Pause<A> {
    a: PhantomData<A>,
}

pub fn pause<A> () -> Pause<A>
where A: 'static {
    Pause {
        a: PhantomData
    }
}

impl<A> Arrow<A,A> for Pause<A>
where A: 'static
{
    
    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<A> {
        rt.on_next_instant (Box::new (move |rt: &mut Runtime, ()| {
            next.call (rt, a);
        }));
    }

}

//  _____ _                  _       _   
// |  ___(_)_  ___ __   ___ (_)_ __ | |_ 
// | |_  | \ \/ / '_ \ / _ \| | '_ \| __|
// |  _| | |>  <| |_) | (_) | | | | | |_ 
// |_|   |_/_/\_\ .__/ \___/|_|_| |_|\__|
//              |_|                      

pub struct Fixpoint<X> {
    arr: Arc<X>,
}

pub fn fixpoint<A,B,X> (x: X) -> Fixpoint<X>
where X: Arrow<A,Result<A,B>> {
    Fixpoint {arr: Arc::new(x)}
}

fn fixpoint_rec<A,B,X,F> (arr: Arc<X>, rt: &mut Runtime, a: A, next: F)
where A: 'static,
      B: 'static,
      X: Arrow<A,Result<A,B>> + 'static,
      F: Continuation<B>
{
    let rec = arr.clone ();
    (*arr).call (rt, a, move |rt: &mut Runtime, r: Result<A,B>| {
        match r {
            Result::Ok(a)  => { fixpoint_rec (rec, rt, a, next); }
            Result::Err(b) => { next.call (rt, b); }
        }
    });
}

impl<A,B,X> Arrow<A,B> for Fixpoint<X>
where A: 'static,
      B: 'static,
      X: Arrow<A,Result<A,B>> + 'static
{

    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<B> {
        fixpoint_rec (self.arr.clone (), rt, a, next);
    }

}

//  ____                _            _   
// |  _ \ _ __ ___   __| |_   _  ___| |_ 
// | |_) | '__/ _ \ / _` | | | |/ __| __|
// |  __/| | | (_) | (_| | |_| | (__| |_ 
// |_|   |_|  \___/ \__,_|\__,_|\___|\__|
//                                       

pub struct Product<X,Y> {
    fst: Arc<X>,
    snd: Arc<Y>,
}

pub fn product<A,B,C,D,X,Y> (x: X, y: Y) -> Product<X,Y>
where A: 'static,
      B: 'static,
      C: 'static,
      D: 'static,
      X: Arrow<A,B> + 'static,
      Y: Arrow<C,D> + 'static,
{
    Product {
        fst: Arc::new (x),
        snd: Arc::new (y),
    }
}

enum ProductJoin<A,B> {
    NoValue,
    ValueA (A),
    ValueB (B),
}

impl<A,B,C,D,X,Y> Arrow<(A,B),(C,D)> for Product<X,Y>
where A: 'static,
      B: 'static,
      C: 'static,
      D: 'static,
      X: Arrow<A,C> + 'static,
      Y: Arrow<B,D> + 'static,
{

    fn call<F> (&self, rt: &mut Runtime, (a,b): (A,B), next: F)
    where F: Continuation<(C,D)> {
        let join_a = Arc::new (Mutex::new (ProductJoin::NoValue));
        let join_b = join_a.clone ();
        let next_a = Arc::new (RefCell::new (Option::Some (next)));
        let next_b = next_a.clone ();
        let fst = self.fst.clone ();
        let snd = self.snd.clone ();
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            (*fst).call (rt, a, move |rt: &mut Runtime, c:C| {
                let mut join = join_a.lock ().unwrap ();
                let mut temp = ProductJoin::NoValue;
                swap (&mut *join, &mut temp);
                match temp {
                    ProductJoin::NoValue => {
                        temp = ProductJoin::ValueA (c);
                        swap (&mut *join, &mut temp);
                    },
                    ProductJoin::ValueA (_) => { panic!(); },
                    ProductJoin::ValueB (d) => {
                        let mut next = next_a.borrow_mut ();
                        let mut temp = Option::None;
                        swap (&mut *next, &mut temp);
                        temp.unwrap ().call (rt, (c,d));
                    },
                }
            });
        }));
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            (*snd).call (rt, b, move |rt: &mut Runtime, d:D| {
                let mut join = join_b.lock ().unwrap ();
                let mut temp = ProductJoin::NoValue;
                swap (&mut *join, &mut temp);
                match temp {
                    ProductJoin::NoValue => {
                        temp = ProductJoin::ValueB (d);
                        swap (&mut *join, &mut temp);
                    },
                    ProductJoin::ValueA (c) => {
                        let mut next = next_b.borrow_mut ();
                        let mut temp = Option::None;
                        swap (&mut *next, &mut temp);
                        temp.unwrap ().call (rt, (c,d));
                    },
                    ProductJoin::ValueB (_) => { panic!(); },
                }
            });
        }));
    }

}

//  _____          _    
// |  ___|__  _ __| | __
// | |_ / _ \| '__| |/ /
// |  _| (_) | |  |   < 
// |_|  \___/|_|  |_|\_\
//                      

pub struct Fork<X> {
   arr: Arc<X>,
}

pub fn fork<A,X> (x: X) -> Fork<X>
where A: 'static,
      X: Arrow<A,()> + 'static,
{
    Fork {arr: Arc::new (x)}
}

impl<A,X> Arrow<A,A> for Fork<X>
where A: Clone + 'static,
      X: Arrow<A,()> + 'static,
{

    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<A> {
        let arr = self.arr.clone ();
        let val = a.clone ();
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            arr.call (rt, val, |_: &mut Runtime, ()| {});
        }));
        next.call (rt, a);
    }

}

