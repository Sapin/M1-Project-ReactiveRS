
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

    //////////////
    // CONSTANT //
    //////////////

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

