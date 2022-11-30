//! ShekozWallet is a simple wallet by @shekohex

use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};

mod config;
mod erc20;
mod qrscanner;
mod state;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    inquire::set_global_render_config(get_render_config());
    let config = config::try_load_or_create_default()?;
    // State machine that will track the current state of the application
    let state = state::AppState::new(config)
        .select_network()?
        .maybe_import_account()?
        .ask_for_operation()?
        .execute()
        .await?;
    // Save the config to disk
    config::save(&state.config)?;
    Ok(())
}

fn get_render_config() -> RenderConfig {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("❯").with_fg(Color::LightBlue);
    render_config.answered_prompt_prefix =
        Styled::new("✔").with_fg(Color::LightGreen);
    render_config.canceled_prompt_indicator =
        Styled::new("✘").with_fg(Color::LightRed);
    render_config.highlighted_option_prefix =
        Styled::new("▶").with_fg(Color::LightYellow);
    render_config.selected_checkbox =
        Styled::new("✔").with_fg(Color::LightGreen);
    render_config.scroll_up_prefix = Styled::new("▲").with_fg(Color::LightBlue);
    render_config.scroll_down_prefix =
        Styled::new("▼").with_fg(Color::LightBlue);
    render_config.unselected_checkbox = Styled::new("☐");

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("✘").with_fg(Color::LightRed));

    render_config.answer = StyleSheet::new()
        .with_attr(Attributes::ITALIC)
        .with_fg(Color::LightBlue);

    render_config.help_message = StyleSheet::new().with_fg(Color::DarkYellow);

    render_config
}
