
use std::sync::{Arc};
use std::cell::{RefCell};
use std::option::{Option};
use std::mem::{swap};
use std::marker::{PhantomData};

use runtime::{Runtime,Continuation};

pub mod prim;

//     _                           
//    / \   _ __ _ __ _____      __
//   / _ \ | '__| '__/ _ \ \ /\ / /
//  / ___ \| |  | | | (_) \ V  V / 
// /_/   \_\_|  |_|  \___/ \_/\_/  
//                                 

pub trait Arrow<'a,A,B> : 'a
where A : 'a,
      B : 'a
{

    fn call<C> (&self, rt: &mut Runtime, a: A, next: C)
    where C: Continuation<B>;

    fn execute (self, a: A) -> B
    where Self: Sized + 'a
    {
        let mut rt = Runtime::new ();
        let val = Arc::new (RefCell::new (Option::None));
        let back = val.clone ();
        rt.on_current_instant (Box::new (move |rt: &mut Runtime, ()| {
            self.call (rt, a, move |_:&mut Runtime, b: B| {
                *(*back).borrow_mut () = Option::Some (b);
            })
        }));
        rt.execute ();

        let mut tmp = Option::None;
        swap (&mut *val.borrow_mut (), &mut tmp);
        match tmp {
            Option::None => panic! (),
            Option::Some (b) => b
        }
    }

    fn bind<C,Y> (self, y: Y) -> Bind<B,Self,Y>
    where Self: Sized + 'a,
          Y: Arrow<'a,B,C>
    {
        bind (self, y)
    }

    fn flatten<C> (self) -> Flatten<Self,B>
    where Self: Sized + 'a,
          B: Arrow<'a,(),C>
    {
        flatten (self)
    }

}

impl<'a,A,B,F> Arrow<'a,A,B> for F
where F: Fn(A) -> B + 'a,
      A : 'a,
      B : 'a
{
    
    fn call<C> (&self, rt: &mut Runtime, a: A, next: C)
    where C: Continuation<B> {
        next.call (rt, self (a))
    }

}

//  ____  _           _ 
// | __ )(_)_ __   __| |
// |  _ \| | '_ \ / _` |
// | |_) | | | | | (_| |
// |____/|_|_| |_|\__,_|
//                      

pub struct Bind<B,X,Y> {
    mid : PhantomData<B>,
    fst : X,
    snd : Arc<Y>,
}

pub fn bind<'a,A,B,C,X,Y> (x: X, y: Y) -> Bind<B,X,Y>
where A : 'a,
      B : 'a,
      C : 'a,
      X: Arrow<'a,A,B>,
      Y: Arrow<'a,B,C>
{
    Bind {
        mid: PhantomData,
        fst: x,
        snd: Arc::new(y),
    }
}


impl<'a,A,B,C,X,Y> Arrow<'a,A,C> for Bind<B,X,Y>
where A : 'a,
      B : 'a,
      C : 'a,
      X: Arrow<'a,A,B>,
      Y: Arrow<'a,B,C>
{
    
    fn call<F> (&self, rt: &mut Runtime, a:A, next:F)
    where F: Continuation<C> {
        let snd = self.snd.clone ();
        self.fst.call (rt, a, move |rt: &mut Runtime, b: B| {
            (*snd).call (rt, b, next);
        });
    }

}

//  _____ _       _   _             
// |  ___| | __ _| |_| |_ ___ _ __  
// | |_  | |/ _` | __| __/ _ \ '_ \ 
// |  _| | | (_| | |_| ||  __/ | | |
// |_|   |_|\__,_|\__|\__\___|_| |_|
//                                  

pub struct Flatten<X,Y> {
    fst: X,
    snd: PhantomData<Y>,
}

pub fn flatten<'a,A,B,X,Y> (arr: X) -> Flatten<X,Y>
where A : 'a,
      B : 'a,
      X: Arrow<'a, A,Y>,
      Y: Arrow<'a,(),B>
{
    Flatten {
        fst: arr,
        snd: PhantomData,
    }
}


impl<'a,A,B,X,Y> Arrow<'a,A,B> for Flatten<X,Y>
where A : 'a,
      B : 'a,
      X: Arrow<'a, A,Y>,
      Y: Arrow<'a,(),B>
{
    
    fn call<F> (&self, rt: &mut Runtime, a: A, next: F)
    where F: Continuation<B> {
        self.fst.call (rt, a, move |rt: &mut Runtime, snd:Y| {
            snd.call (rt, (), next);
        });
    }

}

