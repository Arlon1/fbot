use serde::Deserialize;

#[derive(Deserialize)]
pub struct BotConfig {
    pub foo: String,
    pub bots: BotConfigBots,
}

#[derive(Deserialize)]
pub struct BotConfigBots {
    pub channel: Vec<String>,
}
