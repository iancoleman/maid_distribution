use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const OMNI_BALANCES_URL: &str = "https://api.omniexplorer.info/ask.aspx?api=getpropertybalances&prop=3";
const CACHE_DIR: &str = "cache";
const CACHE_EXPIRY_SECS: u64 = 3600;

#[derive(Serialize, Deserialize)]
struct OMaidBalance {
    address: String,
    balance: String,
    reserved: String,
}

fn main() {
    // omaid
    println!("Fetching omni balances");
    let omaid_balances = fetch_omni_balances();
    println!("Total OMaid Balances: {}", omaid_balances.len());
    // get public keys for addresses
    // generate cashnotes for public keys
    // upload cashnotes to network
}

fn fetch_omni_balances() -> Vec<OMaidBalance> {
    let obody = fetch_from_cache_or_internet(OMNI_BALANCES_URL);
    // parse omni balances
    let obalances: Vec<OMaidBalance> = serde_json::from_str(&obody).unwrap();
    obalances
}

fn fetch_from_cache_or_internet(url: &str) -> String {
    // make directory for caching responses
    let _ = fs::create_dir(CACHE_DIR);
    // check if the url exists in the cache
    let cached_body = get_cached_response(url);
    if cached_body.len() > 0 {
        return cached_body;
    }
    // make the request from the internet
    let body = fetch_from_internet(url);
    // save response body to cache
    save_response_to_cache(url, body.clone());
    body
}

fn fetch_from_internet(url: &str) -> String {
    println!("Fetching {}", url);
    let response = minreq::get(url).send().unwrap();
    response.as_str().unwrap().to_string()
}

fn cache_filename(url: &str) -> PathBuf {
    let url_hash = sha256::digest(url);
    Path::new(CACHE_DIR).join(url_hash)
}

fn get_cached_response(url: &str) -> String {
    let filename = cache_filename(url);
    let metadata = match fs::metadata(filename.clone()) {
        Ok(m) => m,
        Err(_) => return "".to_string(),
    };
    // file is a directory, should never happen
    if metadata.is_dir() {
        fs::remove_dir_all(filename).unwrap();
        return "".to_string();
    }
    // file is too old
    let modified_time = metadata.modified().unwrap();
    let age = SystemTime::now().duration_since(modified_time).unwrap();
    if age.as_secs() > CACHE_EXPIRY_SECS {
        println!("Cache expired: {:?}", url);
        fs::remove_file(filename).unwrap();
        return "".to_string();
    }
    // read from cache
    let mut file = fs::File::open(filename).unwrap();
    let mut body = String::new();
    file.read_to_string(&mut body).unwrap();
    body
}

fn save_response_to_cache(url: &str, body: String) {
    let filename = cache_filename(url);
    // write to cache
    let mut file = fs::File::create(filename).unwrap();
    file.write_all(body.as_bytes()).unwrap();
}
