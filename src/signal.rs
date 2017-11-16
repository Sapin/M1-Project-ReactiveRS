
use std::rc::Rc;
use std::collections::VecDeque;
use runtime::{Runtime,Continuation};
use process::{Process,ProcessMut};

/// A shared pointer to signal runtime
#[derive(Clone)]
pub struct SignalRuntimeRef {
    runtime: Rc<SignalRuntime>,
}

/// Runtime for pure signals
struct SignalRuntime {
    emitted: bool,
    waiters: VecDeque<Box<Continuation<()>>>,
}

impl SignalRuntimeRef {
    /// Sets the signal as emitted for the current instant
    fn emits(self, runtime: &mut Runtime) {
        (*self.runtime).emitted = true;
        while let Some (c) = self.waiters.pop_front() {
            c.call()
        }

        let sr = self.clone();
        runtime.on_end_of_instant(|_, ()| {
            sr.emitted = false
        })
    }

    /// Calls `c` at the first cycle where the signal is present 
    fn on_signal<C>(self, runtime: &mut Runtime, c: C)
    where C: Continuation<()> {
        let sr = self.clone();
        runtime.on_end_of_instant(|rt, ()| {
            if sr.emitted {
                c.call()
            } else {
                sr.waiters.push_back(Box::new(c))
            }
        })
    }
}

/// A reactive signal.
pub trait Signal {
    /// Returns a reference to the signal's runtime.
    fn runtime(self) -> SignalRuntimeRef;

    /// Returns a process that waits for the next emission of the signal, current instant
    /// included.
    fn await_immediate(self) -> AwaitImmediate where Self: Sized {
      unimplemented!() // TODO
    }

    // TODO: add other methods if needed.
}

struct AwaitImmediate {
    // TODO
}

impl Process for AwaitImmediate {
    // TODO
}

impl ProcessMut for AwaitImmediate {
    // TODO
}

