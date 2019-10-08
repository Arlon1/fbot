use serde::Deserialize;

#[derive(Deserialize)]
pub struct BotConfig {
    pub bots: BotConfigBots,
    pub account: BotConfigAccount,
}

#[derive(Deserialize)]
pub struct BotConfigBots {
    pub channel: Vec<String>,
}

#[derive(Deserialize)]
pub struct BotConfigAccount {
    pub user: String,
    pub pass: String,
}
