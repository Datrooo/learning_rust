// use std::{env, fmt::Error};

// pub struct Config {
//     pub region: String,
//     pub access_key_id: String,
//     pub secret_access_key: String,
//     pub endpoint_url: String,
// }

// impl Config {
//     pub fn from_env() -> Result<Self, Box<Error>> {
//         let region = env::var("RUSTFS_REGION")?;
//         let access_key_id = env::var("RUSTFS_ACCESS_KEY_ID")?;
//         let secret_access_key = env::var("RUSTFS_SECRET_ACCESS_KEY")?;
//         let endpoint_url = env::var("RUSTFS_ENDPOINT_URL")?;

//         Ok(Config {
//             region,
//             access_key_id,
//             secret_access_key,
//             endpoint_url,
//         })
//     }
// }
