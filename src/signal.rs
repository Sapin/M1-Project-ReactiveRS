
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::VecDeque;
use runtime::{Runtime,Continuation};
use process::{Process,ProcessMut};

/// A shared pointer to signal runtime
#[derive(Clone)]
pub struct SignalRuntimeRef {
    runtime: Arc<RefCell<SignalRuntime>>,
}

/// Runtime for pure signals
struct SignalRuntime {
    emitted: bool,
    waiters: VecDeque<Box<Continuation<()>>>,
}

impl SignalRuntimeRef {
    /// Sets the signal as emitted for the current instant
    pub fn emits(self, runtime: &mut Runtime) {
        let mut srt = (*self.runtime).try_borrow_mut ();
        while srt.is_err () { srt = (*self.runtime).try_borrow_mut (); };
        let mut srt = srt.unwrap ();
        srt.emitted = true;
        while let Some (c) = srt.waiters.pop_front() {
            c.call_box(runtime, ())
        }

        let sr = self.clone();
        runtime.on_end_of_instant(Box::new(move |_rt : &mut Runtime, ()| {
            let mut srt = (*sr.runtime).try_borrow_mut ();
            while srt.is_err () { srt = (*sr.runtime).try_borrow_mut (); };
            srt.unwrap ().emitted = false
        }))
    }

    /// Calls `c` at the first cycle where the signal is present 
    pub fn on_signal<C>(self, runtime: &mut Runtime, c: C)
    where C: Continuation<()> {
        let sr = self.clone();
        runtime.on_current_instant(Box::new(move |rt : &mut Runtime, ()| {
            let mut srt = (*sr.runtime).try_borrow_mut ();
            while srt.is_err () { srt = (*sr.runtime).try_borrow_mut (); };
            let mut srt = srt.unwrap ();
            if srt.emitted {
                c.call(rt, ())
            } else {
                srt.waiters.push_back(Box::new(c))
            }
        }))
    }
}

/// A reactive signal.
pub trait Signal {
    /// Returns a reference to the signal's runtime.
    fn runtime(self) -> SignalRuntimeRef;

    /// Returns a process that waits for the next emission of the signal, current instant
    /// included.
    fn await_immediate(self) -> AwaitImmediate where Self: Sized {
        AwaitImmediate {signal: self.runtime()}
    }

    // TODO: add other methods if needed.
}

pub struct AwaitImmediate {
    signal : SignalRuntimeRef,
}

impl Process for AwaitImmediate {
    type Value = ();

    fn call<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<Self::Value> {
        self.signal.on_signal(runtime, next)
    }
}

impl ProcessMut for AwaitImmediate {
    fn call_mut<C> (self, runtime: &mut Runtime, next: C)
    where C: Continuation<(Self, Self::Value)> {
        let sig = self.signal.clone ();
        self.signal.on_signal(runtime, next.map(move |()| (
            AwaitImmediate {signal: sig}, ()
        )))
    }
}

