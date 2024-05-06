use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use futures::{channel::mpsc, Stream};
use futures_signals::signal::{Mutable, Signal};

pub trait EventHandler<T>: 'static {
    fn send(&mut self, x: T);

    fn with<G, U>(self, g: G) -> impl EventHandler<U>
    where
        G: FnMut(U) -> T + 'static;

    fn filtered<P>(self, predicate: P) -> impl EventHandler<T>
    where
        P: FnMut(&T) -> bool + 'static;

    fn with_filtered<G, U>(self, g: G) -> impl EventHandler<U>
    where
        G: FnMut(U) -> Option<T> + 'static;
}

impl<F, T> EventHandler<T> for F
where
    F: FnMut(T) + 'static,
{
    fn send(&mut self, x: T) {
        self(x)
    }

    fn with<G, U>(mut self, mut g: G) -> impl EventHandler<U>
    where
        G: FnMut(U) -> T + 'static,
    {
        move |x| self(g(x))
    }

    fn filtered<P>(mut self, mut predicate: P) -> impl EventHandler<T>
    where
        P: FnMut(&T) -> bool + 'static,
    {
        move |x| {
            if predicate(&x) {
                self(x)
            }
        }
    }

    fn with_filtered<G, U>(mut self, mut g: G) -> impl EventHandler<U>
    where
        G: FnMut(U) -> Option<T> + 'static,
    {
        move |x| {
            if let Some(x) = g(x) {
                self(x)
            }
        }
    }
}

pub struct MultiEventHandler<F, T>(Rc<RefCell<F>>, PhantomData<fn(T)>);

impl<F: EventHandler<T>, T> MultiEventHandler<F, T> {
    pub fn new(f: F) -> Self {
        Self(Rc::new(RefCell::new(f)), PhantomData)
    }

    pub fn sink(&self) -> impl EventHandler<T> + 'static {
        let f = self.0.clone();
        move |x| f.borrow_mut().send(x)
    }
}

pub fn signal_channel<T>(initial: T) -> (impl EventHandler<T>, impl Signal<Item = T> + 'static)
where
    T: Clone + 'static,
{
    let state = Mutable::new(initial);
    let sig = state.signal_cloned();
    (move |value| state.set(value), sig)
}

pub fn stream_channel<T>() -> (impl EventHandler<T>, impl Stream<Item = T> + 'static)
where
    T: 'static,
{
    let (sink, stream) = mpsc::unbounded();
    (
        move |ev| {
            if let Err(e) = sink.unbounded_send(ev) {
                assert!(!e.is_full());
            }
        },
        stream,
    )
}
