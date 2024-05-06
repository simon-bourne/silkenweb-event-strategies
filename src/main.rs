use silkenweb::mount;

pub mod event_handler;
pub mod using_data;
pub mod using_traits;

pub mod common {
    use silkenweb::{
        dom::Dom,
        elements::{html::Button, ElementEvents}, node::element::ParentElement,
    };

    use crate::event_handler::EventHandler;

    pub fn button<D: Dom>(mut handler: impl EventHandler<()>) -> Button<D> {
        silkenweb::elements::html::button().on_click(move |_, _| handler.send(()))
    }
    pub fn reset<D: Dom>(handler: impl EventHandler<()>) -> Button<D> {
        button(handler).text("Reset")
    }
}

fn main() {
    mount("trait_app", using_traits::app());
    mount("signal_app", using_data::signal_app());
    mount("stream_app", using_data::stream_app());
    mount("callback_app", using_data::callback_app());
}
