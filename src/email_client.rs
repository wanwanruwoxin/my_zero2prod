use lettre::{
    Message, SmtpTransport, Transport,
    message::{MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    username: String,
    smtp_transport: SmtpTransport,
}

impl EmailClient {
    pub fn new(username: String, password: SecretString, base_url: &str) -> Self {
        let creds = Credentials::new(username.clone(), password.expose_secret().to_string());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(base_url)
            .unwrap()
            .credentials(creds)
            .build();

        Self {
            smtp_transport: mailer,
            username: username.clone(),
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), lettre::error::Error> {
        let email = Message::builder()
            .from(self.username.parse().unwrap())
            .to(recipient.as_ref().parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(text_content.to_string()))
                    .singlepart(SinglePart::html(html_content.to_string())),
            )?;

        self.smtp_transport.send(&email).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        todo!()
    }
}
