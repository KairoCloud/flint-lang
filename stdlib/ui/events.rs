use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Event {
    Click(ClickEvent),
    Input(InputEvent),
    Change(ChangeEvent),
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    Focus(FocusEvent),
    Blur(FocusEvent),
    Submit(SubmitEvent),
    MouseEnter(MouseEvent),
    MouseLeave(MouseEvent),
    MouseMove(MouseEvent),
    Scroll(ScrollEvent),
}

#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub target: String,
    pub client_x: f64,
    pub client_y: f64,
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub target: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub target: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: String,
    pub code: String,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
}

#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub target: String,
}

#[derive(Debug, Clone)]
pub struct SubmitEvent {
    pub target: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub target: String,
    pub client_x: f64,
    pub client_y: f64,
    pub button: u16,
}

#[derive(Debug, Clone)]
pub struct ScrollEvent {
    pub target: String,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

pub fn on_click(handler: impl Fn(&ClickEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Click(ce) = e {
            handler(ce);
        }
    })
}

pub fn on_input(handler: impl Fn(&InputEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Input(ie) = e {
            handler(ie);
        }
    })
}

pub fn on_change(handler: impl Fn(&ChangeEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Change(ce) = e {
            handler(ce);
        }
    })
}

pub fn on_keydown(handler: impl Fn(&KeyEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::KeyDown(ke) = e {
            handler(ke);
        }
    })
}

pub fn on_keyup(handler: impl Fn(&KeyEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::KeyUp(ke) = e {
            handler(ke);
        }
    })
}

pub fn on_focus(handler: impl Fn(&FocusEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Focus(fe) = e {
            handler(fe);
        }
    })
}

pub fn on_blur(handler: impl Fn(&FocusEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Blur(fe) = e {
            handler(fe);
        }
    })
}

pub fn on_submit(handler: impl Fn(&SubmitEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Submit(se) = e {
            handler(se);
        }
    })
}

pub fn on_mouseenter(handler: impl Fn(&MouseEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::MouseEnter(me) = e {
            handler(me);
        }
    })
}

pub fn on_mouseleave(handler: impl Fn(&MouseEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::MouseLeave(me) = e {
            handler(me);
        }
    })
}

pub fn on_mousemove(handler: impl Fn(&MouseEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::MouseMove(me) = e {
            handler(me);
        }
    })
}

pub fn on_scroll(handler: impl Fn(&ScrollEvent) + Send + Sync + 'static) -> EventHandler {
    Box::new(move |e| {
        if let Event::Scroll(se) = e {
            handler(se);
        }
    })
}

#[macro_export]
macro_rules! on {
    (click: $handler:expr) => { ::flint_ui::on_click($handler) };
    (input: $handler:expr) => { ::flint_ui::on_input($handler) };
    (change: $handler:expr) => { ::flint_ui::on_change($handler) };
    (keydown: $handler:expr) => { ::flint_ui::on_keydown($handler) };
    (keyup: $handler:expr) => { ::flint_ui::on_keyup($handler) };
    (focus: $handler:expr) => { ::flint_ui::on_focus($handler) };
    (blur: $handler:expr) => { ::flint_ui::on_blur($handler) };
    (submit: $handler:expr) => { ::flint_ui::on_submit($handler) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler() {
        let handler = on_click(|e| {
            assert_eq!(e.target, "button");
        });
        
        let event = Event::Click(ClickEvent {
            target: "button".to_string(),
            client_x: 0.0,
            client_y: 0.0,
        });
        
        handler(&event);
    }
}