use crate::config::Config;
use crate::notifier::Notifier;
use crate::templates;
use std::process;

pub async fn run_test_email(config_path: Option<&str>) {
    let config = Config::load_or_exit(config_path);

    println!("Sending test alert to SMTP host {}...", config.smtp_host);
    let notifier = Notifier::new(config.clone());
    let subject = "[DOCKTURE TEST] Test Email Notification";

    let plain_body = format!(
        "Hello!\n\nThis is a test notification from Dockture to verify your SMTP setup is working correctly.\n\n\
         Configuration details:\n\
         - SMTP Host: {}\n\
         - SMTP Port: {}\n\
         - Sender Email: {}\n\
         - Receiver Emails: {}\n\n\
         If you are reading this email, your configuration is valid!\n",
        config.smtp_host,
        config.smtp_port,
        config.sender_email,
        config.receiver_emails.join(", ")
    );

    let meta = vec![
        ("SMTP Host", config.smtp_host.clone()),
        ("SMTP Port", config.smtp_port.to_string()),
        ("Sender Email", config.sender_email.clone()),
        ("Recipients", config.receiver_emails.join(", ")),
    ];

    let html_body = templates::render_html_report(
        "SMTP Setup Verification",
        "Connection Valid",
        "#10b981",
        "#f0fdf4",
        &meta,
        None,
    );

    if let Err(e) = notifier.send_notification("test", subject, &plain_body, &html_body) {
        eprintln!("SMTP Test failed: {}", e);
        process::exit(1);
    }

    println!("Test notification sent successfully to receivers.");
}
