use futures_signals::signal::{Mutable, Signal, SignalExt};
use silkenweb::{
    clone,
    dom::Dom,
    elements::html::{div, Div},
    node::element::{Element, ParentElement},
    value::Sig,
};

use crate::common::{button, reset};

trait Counter: Clone + 'static {
    fn increase(&self);
    fn decrease(&self);
}

impl Counter for Mutable<i32> {
    fn increase(&self) {
        self.replace_with(|previous| *previous + 1);
    }

    fn decrease(&self) {
        self.replace_with(|previous| *previous - 1);
    }
}

fn counter<D: Dom>(event_sink: impl Counter, value: impl Signal<Item = i32> + 'static) -> Div<D> {
    div()
        .child(
            div()
                .class("value")
                .text(Sig(value.map(move |v| v.to_string()))),
        )
        .child(
            button({
                clone!(event_sink);
                move |_| event_sink.increase()
            })
            .text("increase"),
        )
        .child(button(move |_| event_sink.decrease()).text("decrease"))
}

trait ResettableCounter: Counter {
    fn reset(&self);
}

impl ResettableCounter for Mutable<i32> {
    fn reset(&self) {
        self.set(0)
    }
}

fn resettable_counter<D: Dom>(
    event_sink: impl ResettableCounter,
    count_sig: impl Signal<Item = i32> + 'static,
) -> Div<D> {
    div().child(counter(event_sink.clone(), count_sig).child(reset(move |()| event_sink.reset())))
}

pub fn app<D: Dom>() -> Div<D> {
    let counter_state = Mutable::new(0);
    div().child(resettable_counter(counter_state.clone(), counter_state.signal()))
}
