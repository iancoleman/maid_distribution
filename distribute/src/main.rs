use serde::{Deserialize, Serialize};
use std::{env, fs, process};
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
    public_key: Option<String>,
}

fn main() {
    // omaid
    println!("Fetching omni balances");
    let omaid_balances = fetch_omni_balances();
    println!("Total OMaid Balances: {}", omaid_balances.len());
    let pubkey_balances = add_public_keys(omaid_balances);
    println!("Total balances with pubkeys: {}", pubkey_balances.len());
    // TODO
    // generate an encrypted cashnote for public keys using ECIES
    // upload cashnotes to network
    // decide what to do with addresses that have no pubkey available
}

fn fetch_omni_balances() -> Vec<OMaidBalance> {
    // TODO print block height and current time
    // TODO Consider whether to deal with transactions in mempool
    let obody = fetch_from_cache_or_internet(OMNI_BALANCES_URL);
    // TODO save body to web archive
    // TODO save body to safe network
    // parse omni balances
    let obalances: Vec<OMaidBalance> = serde_json::from_str(&obody).unwrap();
    obalances
}

fn add_public_keys(balances: Vec<OMaidBalance>) -> Vec<OMaidBalance> {
    let mut pubkey_balances = Vec::<OMaidBalance>::new();
    // look in directory for files where
    // filename is base56 bitcoin address
    // filecontent is hex public key
    let mut keys_path = env::current_dir().unwrap();
    keys_path.push("keys");
    let metadata = fs::metadata(&keys_path);
    if !metadata.is_ok() || !metadata.unwrap().is_dir() {
        println!("keys directory containing public keys does not exist:");
        println!("{}", keys_path.display());
        process::exit(1);
    }
    // iterate over balances looking for pubkeys
    for balance in balances {
        let mut pk_path = keys_path.clone();
        pk_path.push(&balance.address);
        if !pk_path.exists() {
            continue;
        }
        let mut file = fs::File::open(pk_path).unwrap();
        let mut body = String::new();
        file.read_to_string(&mut body).unwrap();
        let pk_balance = OMaidBalance{
            address: balance.address,
            balance: balance.balance,
            reserved: balance.reserved,
            public_key: Some(body),
        };
        pubkey_balances.push(pk_balance);
    }
    pubkey_balances
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
