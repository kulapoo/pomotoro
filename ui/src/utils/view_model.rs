use leptos::prelude::{ReadSignal, WriteSignal};

pub trait ViewModel {
    type State: Clone + 'static;

    fn new() -> Self;
    fn state(&self) -> ReadSignal<Self::State>;
    fn set_state(&self) -> WriteSignal<Self::State>;
}
