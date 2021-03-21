use futures::{stream::FuturesUnordered, TryStreamExt};
use std::collections::HashSet;
use tracing::{debug, instrument};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TwilioConfig {
    /// Twilio account SID.
    pub(crate) account_sid: String,

    /// Twilio auth token.
    pub(crate) auth_token: String,

    /// The sender of the appointment availability text message.
    pub(crate) sms_from: String,

    /// The receiver of the appointment availability text message.
    pub(crate) sms_to: HashSet<String>,
}

#[derive(typed_builder::TypedBuilder)]
pub(crate) struct Client {
    client: reqwest::Client,
    config: TwilioConfig,
}

impl Client {
    #[instrument(name = "Client::send_to_many", skip(self, message), err)]
    pub(crate) async fn send_to_many(&self, message: &str) -> Result<(), reqwest::Error> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.account_sid
        );

        // send appointment availabilty information to all specified phone numbers concurrently
        self.config
            .sms_to
            .iter()
            .map(|sms_to| {
                debug!(message = "sending", sms_to = sms_to.as_str());
                self.client
                    .post(&url)
                    .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
                    .form(&[
                        ("Body", message),
                        ("From", &self.config.sms_from),
                        ("To", sms_to),
                    ])
                    .send()
            })
            .collect::<FuturesUnordered<_>>()
            .try_for_each_concurrent(None, move |response| async move {
                debug!(?response);
                Ok(())
            })
            .await
    }
}
