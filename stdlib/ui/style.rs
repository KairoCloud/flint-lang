use std::collections::HashMap;

pub type Styles = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Style {
    props: HashMap<String, String>,
}

impl Style {
    pub fn new() -> Self {
        Style {
            props: HashMap::new(),
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.props.insert("color".to_string(), color.to_string());
        self
    }

    pub fn background(mut self, color: &str) -> Self {
        self.props.insert("background".to_string(), color.to_string());
        self
    }

    pub fn background_color(mut self, color: &str) -> Self {
        self.props.insert("background-color".to_string(), color.to_string());
        self
    }

    pub fn font_size(mut self, size: &str) -> Self {
        self.props.insert("font-size".to_string(), size.to_string());
        self
    }

    pub fn font_weight(mut self, weight: &str) -> Self {
        self.props.insert("font-weight".to_string(), weight.to_string());
        self
    }

    pub fn padding(mut self, size: &str) -> Self {
        self.props.insert("padding".to_string(), size.to_string());
        self
    }

    pub fn margin(mut self, size: &str) -> Self {
        self.props.insert("margin".to_string(), size.to_string());
        self
    }

    pub fn width(mut self, size: &str) -> Self {
        self.props.insert("width".to_string(), size.to_string());
        self
    }

    pub fn height(mut self, size: &str) -> Self {
        self.props.insert("height".to_string(), size.to_string());
        self
    }

    pub fn display(mut self, value: &str) -> Self {
        self.props.insert("display".to_string(), value.to_string());
        self
    }

    pub fn flex(mut self) -> Self {
        self.display("flex")
    }

    pub fn flex_row(mut self) -> Self {
        self.display("flex").flex_direction("row")
    }

    pub fn flex_column(mut self) -> Self {
        self.display("flex").flex_direction("column")
    }

    pub fn flex_direction(mut self, dir: &str) -> Self {
        self.props.insert("flex-direction".to_string(), dir.to_string());
        self
    }

    pub fn justify_content(mut self, value: &str) -> Self {
        self.props.insert("justify-content".to_string(), value.to_string());
        self
    }

    pub fn align_items(mut self, value: &str) -> Self {
        self.props.insert("align-items".to_string(), value.to_string());
        self
    }

    pub fn border(mut self, value: &str) -> Self {
        self.props.insert("border".to_string(), value.to_string());
        self
    }

    pub fn border_radius(mut self, value: &str) -> Self {
        self.props.insert("border-radius".to_string(), value.to_string());
        self
    }

    pub fn text_align(mut self, value: &str) -> Self {
        self.props.insert("text-align".to_string(), value.to_string());
        self
    }

    pub fn cursor(mut self, cursor: &str) -> Self {
        self.props.insert("cursor".to_string(), cursor.to_string());
        self
    }

    pub fn position(mut self, pos: &str) -> Self {
        self.props.insert("position".to_string(), pos.to_string());
        self
    }

    pub fn top(mut self, value: &str) -> Self {
        self.props.insert("top".to_string(), value.to_string());
        self
    }

    pub fn left(mut self, value: &str) -> Self {
        self.props.insert("left".to_string(), value.to_string());
        self
    }

    pub fn right(mut self, value: &str) -> Self {
        self.props.insert("right".to_string(), value.to_string());
        self
    }

    pub fn bottom(mut self, value: &str) -> Self {
        self.props.insert("bottom".to_string(), value.to_string());
        self
    }

    pub fn z_index(mut self, value: i32) -> Self {
        self.props.insert("z-index".to_string(), value.to_string());
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.props.insert("opacity".to_string(), value.to_string());
        self
    }

    pub fn transition(mut self, value: &str) -> Self {
        self.props.insert("transition".to_string(), value.to_string());
        self
    }

    pub fn transform(mut self, value: &str) -> Self {
        self.props.insert("transform".to_string(), value.to_string());
        self
    }

    pub fn to_css(&self) -> String {
        self.props.iter()
            .map(|(k, v)| format!("{}: {};", k, v))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for Style {
    fn default() -> Self { Self::new() }
}

pub fn stylesheet() -> Stylesheet {
    Stylesheet::new()
}

pub struct Stylesheet {
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Stylesheet {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(mut self, selector: &str, style: Style) -> Self {
        self.rules.push(StyleRule {
            selector: selector.to_string(),
            style,
        });
        self
    }

    pub fn to_css(&self) -> String {
        self.rules.iter()
            .map(|r| format!("{} {{ {} }}", r.selector, r.style.to_css()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for Stylesheet {
    fn default() -> Self { Self::new() }
}

struct StyleRule {
    selector: String,
    style: Style,
}

#[macro_export]
macro_rules! css {
    ($( $selector:literal { $($prop:ident: $value:literal),* $(,)? } ),* $(,)?) => {
        {
            let mut stylesheet = ::flint_ui::stylesheet();
            $(
                stylesheet = stylesheet.add_rule(
                    $selector,
                    ::flint_ui::Style::new()
                    $( .$prop($value) )*
                );
            )*
            stylesheet.to_css()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style() {
        let s = Style::new()
            .color("red")
            .font_size("16px");
        
        assert!(s.to_css().contains("color: red"));
    }

    #[test]
    def test_flex() {
        let s = Style::new().flex().flex_column();
        assert!(s.to_css().contains("display: flex"));
    }
}