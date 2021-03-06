#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("GET request to TurboVax failed")]
    GetData(#[source] reqwest::Error),

    #[error("failed to parse JSON appointment data")]
    ParseData(#[source] reqwest::Error),

    #[error("failed to send availability text message")]
    SendAvailableMessage(#[source] reqwest::Error),

    #[error("failed to send unavailability text message")]
    SendUnavailableMessage(#[source] reqwest::Error),

    #[error("failed to get short URL")]
    GetShortUrl(#[source] reqwest::Error),

    #[error("failed to get short URL text")]
    GetShortUrlText(#[source] reqwest::Error),

    #[error("failed to compute maximum line length because message is empty")]
    GetMaxMessageLineLength,
}
