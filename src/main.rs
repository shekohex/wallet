//! ShekozWallet is a simple wallet by @shekohex

mod config;
mod erc20;
mod qrscanner;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = config::try_load_or_create_default()?;
    let state = state::WalletState::new(config);
    let state = state.select_network()?;
    qrscanner::capture()?;
    Ok(())
}
