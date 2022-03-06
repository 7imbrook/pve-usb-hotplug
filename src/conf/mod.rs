use serde::Deserialize;
use std::collections::HashMap;

pub fn configure_config() -> Configuration {
    let builder = config::Config::builder().add_source(config::File::new(
        "settings.json",
        config::FileFormat::Json,
    ));

    let config = builder.build().expect("Invaid config");
    config.try_deserialize::<Configuration>().unwrap()
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub device_mapping: HashMap<String, Vec<i32>>,
}
