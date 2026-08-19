use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct ObsConfig {
    #[clap(env, long, default_value = "127.0.0.1")]
    pub obs_address: String,

    #[clap(env, long, default_value = "4455")]
    pub obs_port: u16,

    #[clap(env, long)]
    pub obs_password: Option<String>,

    #[clap(env, long, default_value = "♪%TITLE%/%ARTISTS%")]
    pub format: String,
}

#[derive(Clone, Debug, Parser)]
pub struct Config {
    #[clap(env, long)]
    pub discord_token: String,

    #[clap(flatten)]
    pub obs_config: ObsConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Music {
    pub title: String,
    pub artists: String,
    pub albumart: String,
}
