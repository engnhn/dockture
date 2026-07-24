use crate::config::Config;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn send_email(
    config: &Config,
    alert_type: &str,
    subject: &str,
    plain_body: &str,
    html_body: &str,
) -> Result<(), String> {
    if alert_type != "test" {
        if let Some(ref allowed) = config.email_alerts {
            if !allowed.iter().any(|t| t == alert_type) {
                return Ok(());
            }
        }
    }
    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());

    let builder = if config.smtp_port == 465 {
        SmtpTransport::relay(&config.smtp_host)
            .map_err(|e| format!("SMTP initialization error: {}", e))?
    } else {
        SmtpTransport::starttls_relay(&config.smtp_host)
            .map_err(|e| format!("SMTP initialization error: {}", e))?
    };

    let transport = builder.port(config.smtp_port).credentials(creds).build();

    for recipient in &config.receiver_emails {
        let from_addr = config
            .sender_email
            .parse()
            .map_err(|e| format!("Invalid sender email format: {}", e))?;
        let to_addr = recipient
            .parse()
            .map_err(|e| format!("Invalid receiver email format '{}': {}", recipient, e))?;

        let email = Message::builder()
            .from(from_addr)
            .to(to_addr)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(plain_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| format!("Failed to build email: {}", e))?;

        transport
            .send(&email)
            .map_err(|e| format!("Failed to send email to {}: {}", recipient, e))?;
    }

    Ok(())
}
