use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauyerError {
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

    #[error("User input error: {message}")]
    UserInput { message: String },

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, LauyerError>;

impl From<reqwest::Error> for LauyerError {
    fn from(source: reqwest::Error) -> Self {
        let url = source.url().map_or_else(|| "<unknown>".to_owned(), ToString::to_string);
        Self::Http { source, url }
    }
}
