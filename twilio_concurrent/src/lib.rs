use futures::{stream::FuturesUnordered, StreamExt};
use std::collections::HashSet;
use tracing::{debug, instrument};

#[derive(Debug, serde::Deserialize)]
pub struct TwilioConfig {
    /// Twilio account SID.
    pub account_sid: String,

    /// Twilio auth token.
    pub auth_token: String,

    /// The sender of the appointment availability text message.
    pub sms_from: String,

    /// The receiver of the appointment availability text message.
    pub sms_to: HashSet<String>,
}

#[derive(typed_builder::TypedBuilder)]
pub struct Client {
    client: reqwest::Client,

    /// Twilio account SID
    account_sid: String,

    /// Twilio auth token.
    auth_token: String,

    /// The sender of the appointment availability text message.
    sms_from: String,

    /// The receiver of the appointment availability text message.
    sms_to: HashSet<String>,
}

impl Client {
    #[instrument(name = "Client::send_to_many", skip(self, message), err)]
    pub async fn send_to_many(&self, message: &str) -> Result<(), reqwest::Error> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        // send appointment availabilty information to all specified phone numbers concurrently
        let mut text_msg_futures = self
            .sms_to
            .iter()
            .map(|sms_to| {
                debug!(message = "sending", sms_to = sms_to.as_str());
                self.client
                    .post(&url)
                    .basic_auth(&self.account_sid, Some(&self.auth_token))
                    .form(&[("Body", message), ("From", &self.sms_from), ("To", sms_to)])
                    .send()
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(result_response) = text_msg_futures.next().await {
            let response = result_response?;
            debug!(?response);
        }

        Ok(())
    }
}
