use std::collections::VecDeque;
use std::mem;

//   ____            _   _                   _   _             
//  / ___|___  _ __ | |_(_)_ __  _   _  __ _| |_(_) ___  _ __  
// | |   / _ \| '_ \| __| | '_ \| | | |/ _` | __| |/ _ \| '_ \ 
// | |__| (_) | | | | |_| | | | | |_| | (_| | |_| | (_) | | | |
//  \____\___/|_| |_|\__|_|_| |_|\__,_|\__,_|\__|_|\___/|_| |_|
//                                      

/// A reactive continuation awaiting a value of type `V`. For the sake of
/// simplicity, continuation must be valid on the static lifetime.
pub trait Continuation<V>: 'static {
	/// Calls the continuation.
	fn call(self, runtime: &mut Runtime, value: V);

	/// Calls the continuation. Works even if the continuation is boxed.
	///
	/// This is necessary because the size of a value must be known to unbox it.
	/// It is thus impossible to take the ownership of a `Box<Continuation>`
	/// whitout knowing the underlying type of the `Continuation`.
	fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V);

	/// Creates a new continuation that applies a function to the input value
	/// before calling `Self`.
	fn map<F, V2>(self, map: F) -> Map<Self, F>
	where Self: Sized, F: FnOnce(V2) -> V + 'static {
		Map { continuation: self, map }
	}

	/// Creates a new continuation that waits for the next instant to run the
	/// continuation.
	fn pause (self) -> Paused<Self>
	where Self: Sized {
		Paused { continuation: self }
	}
}

impl<V, F> Continuation<V> for F where F: FnOnce(&mut Runtime, V) + 'static {
	fn call(self, runtime: &mut Runtime, value: V)  {
		self(runtime, value);
	}

	fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V) {
		(*self).call(runtime, value);
	}
}

//  __  __             
// |  \/  | __ _ _ __  
// | |\/| |/ _` | '_ \ 
// | |  | | (_| | |_) |
// |_|  |_|\__,_| .__/ 
//              |_| 

/// A continuation that applies a function before calling another continuation.
pub struct Map<C, F> {
	continuation: C,
	map: F
}

impl<C, F, V1, V2> Continuation<V1> for Map<C, F>
    where C: Continuation<V2>, F: FnOnce(V1) -> V2 + 'static
{
	fn call (self, runtime: &mut Runtime, value: V1) {
		self.continuation.call (runtime, (self.map) (value))
	}

	fn call_box (self : Box<Self>, runtime: &mut Runtime, value: V1) {
		(*self).call (runtime, value)
	}
}

//  ____                          _ 
// |  _ \ __ _ _   _ ___  ___  __| |
// | |_) / _` | | | / __|/ _ \/ _` |
// |  __/ (_| | |_| \__ \  __/ (_| |
// |_|   \__,_|\__,_|___/\___|\__,_|
//   

pub struct Paused<C> {
	continuation: C
}

impl<C, V> Continuation<V> for Paused<C>
	where C: Continuation<V>, V: 'static
{
	fn call (self, runtime: &mut Runtime, value:V) {
		runtime.on_next_instant (Box::new (move |rt : &mut Runtime,()| {
			self.continuation.call (rt, value)
		}))
	}

	fn call_box (self : Box<Self>, runtime: &mut Runtime, value: V) {
		(*self).call (runtime, value)
	}
}

//  ____              _   _                
// |  _ \ _   _ _ __ | |_(_)_ __ ___   ___ 
// | |_) | | | | '_ \| __| | '_ ` _ \ / _ \
// |  _ <| |_| | | | | |_| | | | | | |  __/
// |_| \_\\__,_|_| |_|\__|_|_| |_| |_|\___|
//     

/// Runtime for executing reactive continuations.
pub struct Runtime {
	current_instant : VecDeque <Box<Continuation<()>>>,
	endof_instant : VecDeque <Box<Continuation<()>>>,
	next_instant : VecDeque <Box<Continuation<()>>>,
}

impl Runtime {
	/// Creates a new `Runtime`.
	pub fn new() -> Self { Runtime {
		current_instant : VecDeque::new (),
		endof_instant : VecDeque::new (),
		next_instant : VecDeque::new (),
	}}

	/// Executes instants until all work is completed.
	pub fn execute(&mut self) {
		while self.instant () { }
	}

	/// Executes a single instant to completion. Indicates if more work remains
	/// to be done.
	pub fn instant(&mut self) -> bool {
		while let Some (ct) = self.current_instant.pop_front () {
			Continuation::call_box (ct, self, ())
		};
		while let Some (ct) = self.endof_instant.pop_front () {
			Continuation::call_box (ct, self, ())
		};
		mem::swap (&mut self.current_instant, &mut self.next_instant);
		! (self.current_instant.is_empty ())
	}

	/// Registers a continuation to execute on the current instant.
	pub fn on_current_instant(&mut self, c: Box<Continuation<()>>) {
		self.current_instant.push_back (c)
	}

	/// Registers a continuation to execute at the next instant.
	pub fn on_next_instant(&mut self, c: Box<Continuation<()>>) {
		self.next_instant.push_back (c)
	}

	/// Registers a continuation to execute at the end of the instant. Runtime
	/// calls for `c` behave as if they where executed during the next instant.
	pub fn on_end_of_instant(&mut self, c: Box<Continuation<()>>) {
		self.endof_instant.push_back (c)
	}
}

