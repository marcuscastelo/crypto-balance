#[async_trait::async_trait]
pub trait WebDriverExt {
    async fn goto(&self, url: &str) -> Result<(), fantoccini::error::CmdError>;
}
