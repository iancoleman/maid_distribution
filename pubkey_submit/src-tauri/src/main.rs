// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct MaidDistribution {
    #[serde(with = "serde_bytes")]
    transfer: Vec<u8>,
    #[serde(with = "serde_bytes")]
    secret_key: Vec<u8>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn decrypt_distribution(sk_wif: &str, distribution_hex: &str) -> String {
    let sk = bitcoin::PrivateKey::from_wif(sk_wif).unwrap();
    let md_bytes = hex::decode(distribution_hex).unwrap();
    let decrypted_md_bytes = ecies::decrypt(&sk.to_bytes(), &md_bytes).unwrap();
    // TODO just send the decrypted bytes as hex, don't parse them
    // since the decrypted bytes will be used in a separate step
    let md: MaidDistribution = rmp_serde::from_slice(&decrypted_md_bytes).unwrap();
    serde_json::to_string(&md).unwrap()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![decrypt_distribution])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
