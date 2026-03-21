#![allow(clippy::unused_async)]

use crate::error::Result;

pub async fn start(_host: &str, _port: u16) -> Result<()> {
    Ok(())
}
