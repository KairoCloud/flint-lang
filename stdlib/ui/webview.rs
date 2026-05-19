pub struct WebView {
    url: String,
    title: String,
    width: i32,
    height: i32,
    resizable: bool,
}

impl WebView {
    pub fn new(url: &str) -> Self {
        WebView {
            url: url.to_string(),
            title: "Flint App".to_string(),
            width: 800,
            height: 600,
            resizable: true,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn run(&self) {
        println!("Opening webview: {} ({}x{})", self.url, self.width, self.height);
        println!("Title: {}", self.title);
        println!("Resizable: {}", self.resizable);
    }

    pub fn eval(&self, js: &str) {
        println!("Evaluating JS: {}", js);
    }
}

pub fn window(url: &str) -> WebView {
    WebView::new(url)
}

pub fn dialog(message: &str) -> Option<String> {
    println!("Dialog: {}", message);
    Some(String::new())
}

pub fn alert(message: &str) {
    println!("Alert: {}", message);
}

pub fn confirm(message: &str) -> bool {
    println!("Confirm: {}", message);
    true
}

pub fn prompt(message: &str, default: &str) -> Option<String> {
    println!("Prompt: {} (default: {})", message, default);
    Some(default.to_string())
}

pub fn open_file(filters: Vec<FileFilter>) -> Option<String> {
    println!("Open file dialog");
    None
}

pub fn save_file(filters: Vec<FileFilter>, default_name: &str) -> Option<String> {
    println!("Save file dialog: {}", default_name);
    None
}

#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

impl FileFilter {
    pub fn new(name: &str, extensions: Vec<&str>) -> Self {
        FileFilter {
            name: name.to_string(),
            extensions: extensions.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub fn images() -> FileFilter {
    FileFilter::new("Images", vec!["png", "jpg", "jpeg", "gif", "webp"])
}

pub fn text_files() -> FileFilter {
    FileFilter::new("Text Files", vec!["txt", "md", "json", "xml"])
}

pub fn code_files() -> FileFilter {
    FileFilter::new("Code", vec!["rs", "js", "ts", "py", "flint"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview() {
        let wv = WebView::new("http://localhost:3000")
            .title("My App")
            .size(1024, 768);
        
        assert_eq!(wv.width, 1024);
    }
}