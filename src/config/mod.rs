use config::{Config, File};

pub fn load_oz_server_config() -> Config {
    Config::builder()
        .add_source(File::with_name(".config.json"))
        .build()
        .unwrap()
}


lazy_static::lazy_static! {
    pub static ref OZ_SERVER_CONFIG: Config = load_oz_server_config();
}

