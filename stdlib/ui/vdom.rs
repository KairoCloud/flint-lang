use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum VNode {
    Element(VElement),
    Text(VText),
    Component(VComponent),
    Fragment(VFragment),
    Empty,
}

#[derive(Debug, Clone)]
pub struct VElement {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub styles: HashMap<String, String>,
    pub events: HashMap<String, EventHandler>,
    pub children: Vec<VNode>,
    pub key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VText {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct VComponent {
    pub name: String,
    pub props: HashMap<String, String>,
    pub children: Vec<VNode>,
}

#[derive(Debug, Clone)]
pub struct VFragment {
    pub children: Vec<VNode>,
}

impl VText {
    pub fn new(content: String) -> Self {
        VText { content }
    }
}

impl VElement {
    pub fn new(tag: &str) -> Self {
        VElement {
            tag: tag.to_string(),
            attrs: HashMap::new(),
            styles: HashMap::new(),
            events: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_string(), value.to_string());
        self
    }

    pub fn style(mut self, key: &str, value: &str) -> Self {
        self.styles.insert(key.to_string(), value.to_string());
        self
    }

    pub fn on(mut self, event: &str, handler: EventHandler) -> Self {
        self.events.insert(event.to_string(), handler);
        self
    }

    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<VNode>) -> Self {
        self.children = children;
        self
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    pub fn text(self, content: &str) -> Self {
        self.child(VNode::Text(VText::new(content.to_string())))
    }

    pub fn class(self, class: &str) -> Self {
        self.attr("class", class)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }
}

impl VNode {
    pub fn element(tag: &str) -> VElement {
        VElement::new(tag)
    }

    pub fn text(content: &str) -> VNode {
        VNode::Text(VText::new(content.to_string()))
    }

    pub fn component(name: &str) -> VComponent {
        VComponent {
            name: name.to_string(),
            props: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn fragment(children: Vec<VNode>) -> VNode {
        VNode::Fragment(VFragment { children })
    }

    pub fn empty() -> VNode {
        VNode::Empty
    }
}

pub fn div() -> VElement { VElement::new("div") }
pub fn span() -> VElement { VElement::new("span") }
pub fn p() -> VElement { VElement::new("p") }
pub fn h1() -> VElement { VElement::new("h1") }
pub fn h2() -> VElement { VElement::new("h2") }
pub fn h3() -> VElement { VElement::new("h3") }
pub fn h4() -> VElement { VElement::new("h4") }
pub fn h5() -> VElement { VElement::new("h5") }
pub fn h6() -> VElement { VElement::new("h6") }
pub fn a() -> VElement { VElement::new("a") }
pub fn button() -> VElement { VElement::new("button") }
pub fn input() -> VElement { VElement::new("input") }
pub fn textarea() -> VElement { VElement::new("textarea") }
pub fn select() -> VElement { VElement::new("select") }
pub fn option() -> VElement { VElement::new("option") }
pub fn img() -> VElement { VElement::new("img") }
pub fn ul() -> VElement { VElement::new("ul") }
pub fn ol() -> VElement { VElement::new("ol") }
pub fn li() -> VElement { VElement::new("li") }
pub fn table() -> VElement { VElement::new("table") }
pub fn tr() -> VElement { VElement::new("tr") }
pub fn th() -> VElement { VElement::new("th") }
pub fn td() -> VElement { VElement::new("td") }
pub fn form() -> VElement { VElement::new("form") }
pub fn label() -> VElement { VElement::new("label") }
pub fn script() -> VElement { VElement::new("script") }
pub fn style() -> VElement { VElement::new("style") }
pub fn link() -> VElement { VElement::new("link") }
pub fn meta() -> VElement { VElement::new("meta") }
pub fn head() -> VElement { VElement::new("head") }
pub fn body() -> VElement { VElement::new("body") }
pub fn html() -> VElement { VElement::new("html") }

pub fn render_to_string(node: &VNode) -> String {
    match node {
        VNode::Element(e) => render_element(e),
        VNode::Text(t) => t.content.clone(),
        VNode::Component(c) => format!("<{}/>", c.name),
        VNode::Fragment(f) => f.children.iter().map(render_to_string).collect(),
        VNode::Empty => String::new(),
    }
}

fn render_element(el: &VElement) -> String {
    let mut html = format!("<{}", el.tag);
    
    for (k, v) in &el.attrs {
        html.push_str(&format!(" {}=\"{}\"", k, v));
    }
    
    if !el.styles.is_empty() {
        let style_str: String = el.styles.iter()
            .map(|(k, v)| format!("{}: {};", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        html.push_str(&format!(" style=\"{}\"", style_str));
    }
    
    html.push('>');
    
    for child in &el.children {
        html.push_str(&render_to_string(child));
    }
    
    html.push_str(&format!("</{}>", el.tag));
    html
}

pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_inner(&mut patches, "", old, new);
    patches
}

fn diff_inner(patches: &mut Vec<Patch>, path: &str, old: &VNode, new: &VNode) {
    match (old, new) {
        (VNode::Text(t1), VNode::Text(t2)) if t1.content != t2.content => {
            patches.push(Patch::TextChange(path.to_string(), t2.content.clone()));
        }
        (VNode::Element(e1), VNode::Element(e2)) if e1.tag != e2.tag => {
            patches.push(Patch::Replace(path.to_string(), new.clone()));
        }
        (VNode::Element(e1), VNode::Element(e2)) => {
            // Diff attributes and children
            for (k, v) in &e2.attrs {
                if e1.attrs.get(k) != Some(&v.to_string()) {
                    patches.push(Patch::Attr(path.to_string(), k.clone(), v.clone()));
                }
            }
        }
        _ => {
            if old != new {
                patches.push(Patch::Replace(path.to_string(), new.clone()));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Patch {
    Replace(String, VNode),
    TextChange(String, String),
    Attr(String, String, String),
    AddChild(String, VNode),
    RemoveChild(String),
}

pub struct Renderer {
    container: String,
}

impl Renderer {
    pub fn new(element: &str) -> Self {
        Renderer {
            container: element.to_string(),
        }
    }

    pub fn render(&self, node: VNode) {
        let html = render_to_string(&node);
        println!("Rendering to {}: {}", self.container, html);
    }

    pub fn patch(&self, old: VNode, new: VNode) {
        let patches = diff(&old, &new);
        for patch in patches {
            println!("Applying patch: {:?}", patch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velement() {
        let el = div()
            .id("app")
            .class("container")
            .text("Hello");
        
        assert_eq!(el.tag, "div");
    }

    #[test]
    fn test_render() {
        let node = VNode::Element(div().text("Hello"));
        let html = render_to_string(&node);
        assert!(html.contains("<div>"));
    }
}