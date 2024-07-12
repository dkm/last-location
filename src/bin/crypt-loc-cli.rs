use clap::{Arg, Command};
use serde::{Deserialize, Serialize};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Loc {
    pub device_timestamp: i32,

    pub lat: f64,
    pub lon: f64,

    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

async fn load_from_file(filename: &str) -> Vec<Loc> {
    let data_str = std::fs::read_to_string(filename).expect("Unable to read file");
    serde_json::from_str(&data_str).expect("JSON does not have correct format.")
}

async fn crypt_loc(loc: &Loc, cipher: &Aes256Gcm) -> String {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // Use time as the IV.
    // IV should be unique and never reused.
    let sec = since_the_epoch.as_secs();
    let nanos = since_the_epoch.subsec_nanos();

    let mut iv_time_based: Vec<u8> = sec.to_le_bytes().to_vec();
    iv_time_based.append(&mut nanos.to_le_bytes().to_vec());

    // cipher the time based IV to make it impossible to get back the initial
    // time used.
    let init_nonce: &Nonce<_> = Nonce::from_slice(&iv_time_based[0..12]);
    let ciphered_nonce = cipher.encrypt(init_nonce, &iv_time_based[0..12]).unwrap();

    // let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let orig_plain: String = serde_json::to_string(loc).expect("json error"); // arg_text;

    let nonce: &Nonce<_> = Nonce::from_slice(&ciphered_nonce[0..12]);

    let ciphertext = cipher.encrypt(nonce, orig_plain.as_ref()).unwrap();

    let iv_ciphered_hex = format!("{}{}", hex::encode(nonce), hex::encode(&ciphertext));
    iv_ciphered_hex
}

async fn send_crypt_loc(
    url: &str,
    crypt_loc: &str,
    priv_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = [("priv_token", priv_token), ("data", crypt_loc)];
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{}/api/s/set_last_location", url))
        .form(&params)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("crypto-last-loc")
        .version("0.1")
        .author("Marc Poulhiès <dkm@kataplop.net>")
        .about("lastloc client")
        .arg(
            Arg::new("url")
                .long("url")
                .default_value("http://localhost:3000"),
        )
        .arg(Arg::new("token").long("token").required(true))
        .arg(Arg::new("secret-key").long("secret-key").required(true))
        .arg(Arg::new("data").long("data").required(true))
        .get_matches();

    let arg_key = matches
        .get_one::<String>("secret-key")
        .ok_or("can't happen")?;

    let url = matches.get_one::<String>("url").ok_or("can't happen")?;

    let arg_token = matches.get_one::<String>("token").ok_or("can't happen")?;

    let arg_data = matches.get_one::<String>("data").expect("can't be missing");

    let data = load_from_file(arg_data).await; //    .expect("can't load from file");

    let key: Vec<u8> = hex::decode(arg_key).expect("key not in hexa");
    let b: &[u8] = &key;
    let key: &Key<Aes256Gcm> = b.into();

    let cipher = Aes256Gcm::new(key);

    for loc in data {
        let c = crypt_loc(&loc, &cipher).await;
        send_crypt_loc(url, &c, arg_token).await?;
    }

    Ok(())
}
