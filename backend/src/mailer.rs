//! Email delivery through SMTP (Gmail with an app password by default).

use anyhow::{bail, Context};
use lettre::{
    message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::settings::Settings;

pub struct Mail {
    pub subject: String,
    pub text: String,
    pub html: String,
    /// iCalendar attachment: (file name, content).
    pub ics: Option<(String, String)>,
}

fn transport(s: &Settings) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    if s.smtp_host.is_empty() || s.smtp_username.is_empty() || s.smtp_password.is_empty() {
        bail!("configura server SMTP, utente e password");
    }
    let builder = if s.smtp_port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&s.smtp_host)?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&s.smtp_host)?
    };
    Ok(builder
        .port(s.smtp_port)
        .credentials(Credentials::new(s.smtp_username.clone(), s.smtp_password.clone()))
        .timeout(Some(std::time::Duration::from_secs(20)))
        .build())
}

pub async fn send(s: &Settings, mail: Mail) -> anyhow::Result<()> {
    let recipients = s.recipients();
    if recipients.is_empty() {
        bail!("nessun destinatario configurato");
    }
    let from: Mailbox = format!("MyGarage <{}>", s.sender()).parse().context("mittente non valido")?;
    let mut builder = Message::builder().from(from).subject(mail.subject);
    for r in &recipients {
        builder = builder.to(r.parse().with_context(|| format!("destinatario non valido: {r}"))?);
    }

    let body = MultiPart::alternative()
        .singlepart(SinglePart::plain(mail.text))
        .singlepart(SinglePart::html(mail.html));
    let message = match mail.ics {
        Some((name, content)) => {
            let ct = ContentType::parse("text/calendar; charset=utf-8; method=PUBLISH")?;
            builder.multipart(MultiPart::mixed().multipart(body).singlepart(Attachment::new(name).body(content, ct)))?
        }
        None => builder.multipart(body)?,
    };
    transport(s)?.send(message).await?;
    Ok(())
}

pub async fn send_test(s: &Settings, base_url: &str) -> anyhow::Result<()> {
    send(
        s,
        Mail {
            subject: "MyGarage: email di prova".into(),
            text: format!("Le notifiche di MyGarage funzionano.\n{base_url}"),
            html: html_wrap(
                "Email di prova",
                "<p>Le notifiche di MyGarage funzionano.</p>",
                &format!("{base_url}/impostazioni"),
                "Apri MyGarage",
            ),
            ics: None,
        },
    )
    .await
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Simple, email-client-safe layout (tables + inline styles).
pub fn html_wrap(title: &str, body_html: &str, link: &str, link_label: &str) -> String {
    format!(
        r#"<!doctype html><html><body style="margin:0;padding:24px;background:#F4F6FA;font-family:Segoe UI,Helvetica,Arial,sans-serif;color:#0F172A">
<table role="presentation" width="100%" cellspacing="0" cellpadding="0"><tr><td align="center">
<table role="presentation" width="560" cellspacing="0" cellpadding="0" style="max-width:560px;background:#FFFFFF;border:1px solid #E2E8F0;border-radius:16px">
<tr><td style="padding:24px 28px 8px"><div style="font-size:13px;font-weight:700;color:#2563EB;letter-spacing:.04em">MYGARAGE</div>
<h1 style="font-size:22px;margin:8px 0 0;line-height:1.3">{title}</h1></td></tr>
<tr><td style="padding:8px 28px 8px;font-size:15px;line-height:1.6;color:#334155">{body_html}</td></tr>
<tr><td style="padding:8px 28px 28px"><a href="{link}" style="display:inline-block;background:#2563EB;color:#FFFFFF;text-decoration:none;font-weight:700;padding:12px 20px;border-radius:10px">{link_label}</a></td></tr>
</table></td></tr></table></body></html>"#,
        title = html_escape(title),
    )
}
