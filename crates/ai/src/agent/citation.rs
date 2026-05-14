use std::fmt::Display;

/// A citation listed in an AI response.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AIAgentCitation {
    LocalObject { uid: String },
    WarpDocumentation { path: String },
    WebPage { url: String },
}

impl Display for AIAgentCitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIAgentCitation::LocalObject { uid } => {
                write!(f, "Local Object: {uid}")
            }
            AIAgentCitation::WarpDocumentation { path } => {
                write!(f, "Warp Documentation: {path}")
            }
            AIAgentCitation::WebPage { url } => {
                write!(f, "Web Page: {url}")
            }
        }
    }
}
