use std::collections::VecDeque;
use std::mem;

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
}

impl<V, F> Continuation<V> for F where F: FnOnce(&mut Runtime, V) + 'static {
	fn call(self, runtime: &mut Runtime, value: V)  {
		self(runtime, value);
	}

	fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V) {
		(*self).call(runtime, value);
	}
}

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

