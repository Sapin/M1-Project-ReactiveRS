
use std::rc::{Rc};
use std::marker::{PhantomData};

//     _                           
//    / \   _ __ _ __ _____      __
//   / _ \ | '__| '__/ _ \ \ /\ / /
//  / ___ \| |  | | | (_) \ V  V / 
// /_/   \_\_|  |_|  \___/ \_/\_/  
//                                 

pub trait Arrow<'a,A,B>
where A : 'a,
      B : 'a
{
    fn bind<C,Y> (self, y: Y) -> Bind<B,Self,Y>
    where Self: Sized + 'a,
          Y: Arrow<'a,B,C>
    {
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
    snd : Rc<Y>,
}

