#![forbid(unsafe_code)]

fn main() -> anyhow::Result<()> {
    rectiq_cli::security::redact::init_tracing();
    rectiq_cli::security::redact::install_panic_hook();
    let args = std::env::args();
    rectiq_cli::run_with_args(args)
}
