
//  ____                              
// |  _ \ _ __ ___   ___ ___  ___ ___ 
// | |_) | '__/ _ \ / __/ _ \/ __/ __|
// |  __/| | | (_) | (_|  __/\__ \__ \
// |_|   |_|  \___/ \___\___||___/___/
//  

use runtime::{Runtime,Continuation} ;

pub trait Process: 'static {
    type Value;
    
    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value>;

    fn value<V> (v : V) -> Constant<V>
    where V: 'static {
        Constant {value : v}
    }

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

