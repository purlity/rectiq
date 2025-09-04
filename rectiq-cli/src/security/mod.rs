// rectiq-cli/src/security/mod.rs

pub mod aad;

/// Install a tracing layer that redacts secrets from logs.
///
/// # Panics
/// This function may panic if tracing subscriber initialization fails.
#[allow(clippy::missing_const_for_fn)]
pub fn install_logging_redactor() {
    #[cfg(all(feature = "secure-default", not(feature = "insecure-dev")))]
    {
        use std::io::{self, Write};
        use tracing_subscriber::{
            filter::LevelFilter,
            fmt,
            layer::{Layer, SubscriberExt},
            util::SubscriberInitExt,
        };

        struct RedactingWriter<W: Write> {
            inner: W,
        }

        impl<W: Write> Write for RedactingWriter<W> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let mut text = String::from_utf8_lossy(buf).into_owned();
                for key in [
                    "Authorization",
                    "X-Ephemeral-Key-Hash",
                    "X-Rectiq-Nonce",
                    "RECTIQ_API_KEY",
                    "X-Rectiq-AAD",
                    "X-Rectiq-Timestamp",
                    "license_key",
                    "ephemeral_key",
                    "ciphertext",
                    "nonce",
                ] {
                    text = text.replace(&format!("{key}: "), &format!("{key}: ***"));
                    text = text.replace(&format!("{key}="), &format!("{key}=***"));
                    text = text.replace(&format!("\"{key}\":\""), &format!("\"{key}\":\"***"));
                }
                self.inner.write(text.as_bytes())
            }

            fn flush(&mut self) -> io::Result<()> {
                self.inner.flush()
            }
        }

        struct MakeRedactingWriter;

        impl<'a> fmt::MakeWriter<'a> for MakeRedactingWriter {
            type Writer = RedactingWriter<io::Stdout>;

            fn make_writer(&'a self) -> Self::Writer {
                RedactingWriter {
                    inner: io::stdout(),
                }
            }
        }

        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .event_format(fmt::format().with_target(false))
                    .with_writer(MakeRedactingWriter)
                    .with_filter(LevelFilter::INFO),
            )
            .init();
    }
}

#[allow(clippy::missing_const_for_fn)]
/// Set a panic hook that strips secrets from panic messages and backtraces.
pub fn install_panic_hook() {
    #[cfg(all(feature = "secure-default", not(feature = "insecure-dev")))]
    {
        std::panic::set_hook(Box::new(|info| {
            let msg = info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
                .unwrap_or("panic");
            let scrubbed = msg
                .replace("Bearer ", "Bearer ***")
                .replace("RECTIQ_API_KEY=", "RECTIQ_API_KEY=***");
            eprintln!("rectiq panic: {scrubbed}");
        }));
    }
}
