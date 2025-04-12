pub enum TextSection {
    String(String),
    Type(String),
}

#[derive(Debug, Clone)]
pub struct Text {
    pub head: String,
    pub note: Option<String>,
}
