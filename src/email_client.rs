use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};

pub struct EmailClient {
    smtp_transport: SmtpTransport,
}

impl EmailClient {
    pub fn new(username: String, password: String, base_url: &str) -> Self {
        let creds = Credentials::new(username, password);

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(base_url)
            .unwrap()
            .credentials(creds)
            .build();

        Self {
            smtp_transport: mailer,
        }
    }

    pub async fn send_email(
        &self,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
