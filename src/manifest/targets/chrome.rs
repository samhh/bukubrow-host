use crate::config::{DESC, NAME};

/// This manifest shape targets Chrome but is used by a number of other Blink-
/// and Webkit-based browsers.
#[derive(Serialize)]
pub struct ChromeHost {
    name: &'static str,
    description: &'static str,
    path: String,
    r#type: &'static str,
    allowed_origins: [&'static str; 1],
}

impl ChromeHost {
    pub fn new<T: Into<String>>(path: T) -> Self {
        ChromeHost {
            name: NAME,
            description: DESC,
            path: path.into(),
            r#type: "stdio",
            allowed_origins: ["chrome-extension://ghniladkapjacfajiooekgkfopkjblpn/"],
        }
    }
}
