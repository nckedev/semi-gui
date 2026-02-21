use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NvimResponse {
    pub version: u32,
    pub filename: String,
    pub filetype: String,
    pub start: u32,
    pub stop: u32,
    pub content: Vec<String>,
}

impl NvimResponse {
    pub fn content_as_md(&self) -> String {
        as_md(&self.content, &self.filetype)
    }
}

fn as_md(content: &Vec<String>, ft: &str) -> String {
    let mut buf = String::default();
    buf.push_str(&format!("```{}", ft));
    buf.push('\n');
    for str in content {
        buf.push_str(str);
        buf.push('\n');
    }
    buf.push_str("```\n");
    buf
}
