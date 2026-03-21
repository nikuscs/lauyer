use thiserror::Error;

#[derive(Debug, Error)]
pub enum LawyerrError {
    #[error("HTTP error fetching '{url}': {source}")]
    Http {
        #[source]
        source: reqwest::Error,
        url: String,
    },

    #[error("Parse error at '{source_url}': {message}")]
    Parse { message: String, source_url: String },

    #[error("Encoding error: {message}")]
    Encoding { message: String },

    #[error("Session error: {message}")]
    Session { message: String },

    #[error("Config error: {message}")]
    Config { message: String },

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, LawyerrError>;

impl From<reqwest::Error> for LawyerrError {
    fn from(source: reqwest::Error) -> Self {
        let url = source.url().map_or_else(|| "<unknown>".to_owned(), ToString::to_string);
        Self::Http { source, url }
    }
}
