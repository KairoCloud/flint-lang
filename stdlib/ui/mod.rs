pub mod component;
pub mod state;
pub mod style;
pub mod vdom;
pub mod events;
pub mod webview;

pub use component::{Component, ComponentBuilder, View};
pub use state::{State, StateManager, derived, effect};
pub use style::{Style, Styles};
pub use vdom::{VNode, VElement, VText, VComponent, render_to_string};
pub use events::{Event, EventHandler, on_click, on_input, on_change};
pub use webview::{WebView, window, dialog, alert, confirm, prompt, open_file, save_file};

pub fn run_app<C: Component>(root: &str) {
    println!("Running UI app at {}", root);
}

pub fn mount<C: Component>(component: C, element: &str) {
    println!("Mounting component to {}", element);
}

#[macro_export]
macro_rules! view {
    ($element:ident { $($content:tt)* }) => {
        // Build virtual DOM node
    };
    (div { $($content:tt)* }) => { /* ... */ };
    (span { $($content:tt)* }) => { /* ... */ };
    (button { $($content:tt)* }) => { /* ... */ };
    (input { $($content:tt)* }) => { /* ... */ };
}

#[macro_export]
macro_rules! style {
    ($($prop:ident: $value:expr),* $(,)?) => {
        ::std::collections::HashMap::from([
            $( (stringify!($prop).to_string(), $value.to_string()) ),*
        ])
    };
}