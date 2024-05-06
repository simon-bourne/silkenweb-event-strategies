use std::future;

use futures::StreamExt;
use futures_signals::signal::{self, Mutable, Signal, SignalExt};
use silkenweb::{
    dom::Dom,
    elements::html::{div, Div},
    node::element::{Element, ParentElement},
    value::Sig,
};

use crate::{
    common::{button, reset},
    event_handler::{signal_channel, stream_channel, EventHandler, MultiEventHandler},
};

enum CounterEvent {
    Increase,
    Decrease,
}

fn counter<D: Dom>(
    handler: impl EventHandler<CounterEvent>,
    value: impl Signal<Item = i32> + 'static,
) -> Div<D> {
    let handler = MultiEventHandler::new(handler);

    div()
        .child(
            div()
                .class("value")
                .text(Sig(value.map(move |v| v.to_string()))),
        )
        .child(button(handler.sink().with(|_| CounterEvent::Increase)).text("increase"))
        .child(button(handler.sink().with(|_| CounterEvent::Decrease)).text("decrease"))
}

enum ResettableCounter {
    Counter(CounterEvent),
    Reset,
}

fn resettable_counter<D: Dom>(
    events: impl EventHandler<ResettableCounter>,
    count_sig: impl Signal<Item = i32> + 'static,
) -> Div<D> {
    let events = MultiEventHandler::new(events);
    div().child(
        counter(events.sink().with(ResettableCounter::Counter), count_sig)
            .child(reset(events.sink().with(|()| ResettableCounter::Reset))),
    )
}

pub fn signal_app<D: Dom>() -> Div<D> {
    let (events, count_sig) = signal_channel(0);
    let mut count = 0;
    let events = events.with(move |event| {
        match event {
            ResettableCounter::Counter(counter) => match counter {
                CounterEvent::Increase => count += 1,
                CounterEvent::Decrease => count -= 1,
            },
            ResettableCounter::Reset => count = 0,
        }

        count
    });

    resettable_counter(events, count_sig)
}

pub fn stream_app<D: Dom>() -> Div<D> {
    let (ev_sink, ev_stream) = stream_channel();
    let counts = signal::from_stream(ev_stream.scan(0, move |count, event| {
        match event {
            ResettableCounter::Counter(counter) => match counter {
                CounterEvent::Increase => *count += 1,
                CounterEvent::Decrease => *count -= 1,
            },
            ResettableCounter::Reset => *count = 0,
        }

        future::ready(Some(*count))
    }))
    .map(|x| x.unwrap_or(0));

    div().child(resettable_counter(ev_sink, counts))
}

pub fn callback_app<D: Dom>() -> Div<D> {
    let count = Mutable::new(0);
    let handler = {
        let count = count.clone();
        move |event: ResettableCounter| match event {
            ResettableCounter::Counter(counter) => {
                match counter {
                    CounterEvent::Increase => count.replace_with(|previous| *previous + 1),
                    CounterEvent::Decrease => count.replace_with(|previous| *previous - 1),
                };
            }
            ResettableCounter::Reset => count.set(0),
        }
    };

    div().child(resettable_counter(handler, count.signal()))
}
