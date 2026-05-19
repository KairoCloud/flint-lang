use std::collections::HashMap;

pub trait Component: Sized + 'static {
    type State: Clone + 'static;
    type Props: Default + 'static;

    fn init() -> Self::State;
    fn render(state: &Self::State, props: &Self::Props) -> VNode;
    fn update(&mut self, state: &mut Self::State);
}

pub struct ComponentBuilder<C: Component> {
    props: C::Props,
    key: Option<String>,
    children: Vec<VNode>,
}

impl<C: Component> ComponentBuilder<C> {
    pub fn new() -> Self {
        ComponentBuilder {
            props: C::Props::default(),
            key: None,
            children: Vec::new(),
        }
    }

    pub fn prop(mut self, prop: C::Props) -> Self {
        self.props = prop;
        self
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> C {
        // In a real implementation, this would instantiate the component
        unimplemented!()
    }
}

impl<C: Component> Default for ComponentBuilder<C> {
    fn default() -> Self { Self::new() }
}

pub trait View {
    fn view(&self) -> VNode;
}

pub struct Fragment {
    children: Vec<VNode>,
}

impl Fragment {
    pub fn new(children: Vec<VNode>) -> Self {
        Fragment { children }
    }
}

pub fn fragment(children: Vec<VNode>) -> VNode {
    VNode::Fragment(Fragment { children })
}

pub fn list<T: Clone>(items: Vec<T>, render: impl Fn(&T) -> VNode) -> VNode {
    let children: Vec<VNode> = items.iter().map(render).collect();
    fragment(children)
}

pub fn cond(condition: bool, then_branch: VNode, else_branch: Option<VNode>) -> VNode {
    if condition {
        then_branch
    } else {
        else_branch.unwrap_or(VNode::Text(VText::new(String::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_builder() {
        let builder = ComponentBuilder::<DummyComponent>::new();
        assert!(builder.key.is_none());
    }

    struct DummyComponent;
    impl Component for DummyComponent {
        type State = ();
        type Props = ();
        fn init() -> Self::State { () }
        fn render(_: &Self::State, _: &Self::Props) -> VNode { VNode::Text(VText::new("".to_string())) }
        fn update(&mut self, _: &mut Self::State) {}
    }
}