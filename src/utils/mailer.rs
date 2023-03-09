use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, response::Response, Error},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct EnvelopeContent {
    pub from: Mailbox,
    pub to: Mailbox,
    pub subject: String,
    pub body: String,
}

pub struct Mailer<'a> {
    pub host_addr: &'a str,
    pub username: String,
    pub password: String,
}



pub async fn send_mail(ec: EnvelopeContent, mailer: Mailer<'_>) -> Result<Response, Error> {
    let email = Message::builder()
        .from(ec.from)
        .to(ec.to)
        .subject(ec.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(ec.body)
        .unwrap();
    let creds = Credentials::new(mailer.username, mailer.password);
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(mailer.host_addr)
            .unwrap()
            .credentials(creds)
            .build();

    mailer.send(email).await
}
