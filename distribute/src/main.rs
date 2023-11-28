use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const OMNI_BALANCES_URL: &str = "https://api.omniexplorer.info/ask.aspx?api=getpropertybalances&prop=3";
const ERC20_BALANCES_URL: &str = "https://etherscan.io/exportData?type=tokenholders&contract=0x329c6E459FFa7475718838145e5e85802Db2a303&decimal=18";
const ERC20_BALANCES_FILENAME: &str = "export-tokenholders-for-contract-0x329c6E459FFa7475718838145e5e85802Db2a303.csv";
const CACHE_DIR: &str = "cache";
const CACHE_EXPIRY_SECS: u64 = 3600;

#[derive(Serialize, Deserialize)]
struct OMaidBalance {
    address: String,
    balance: String,
    reserved: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct EMaidBalance {
    HolderAddress: String,
    Balance: String,
    PendingBalanceUpdate: String,
}

fn main() {
    // omaid
    println!("Fetching omni balances");
    let omaid_balances = fetch_omni_balances();
    println!("Total OMaid Balances: {}", omaid_balances.len());
    // emaid
    println!("Fetching erc20 balances");
    let emaid_balances = fetch_erc20_balances();
    println!("Total EMaid Balances: {}", emaid_balances.len());
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

fn emaid_filename() -> PathBuf {
    let ud = UserDirs::new().unwrap();
    let dd = ud.download_dir().unwrap();
    Path::new(&dd).join(ERC20_BALANCES_FILENAME)
}

fn fetch_erc20_balances() -> Vec<EMaidBalance> {
    // cannot fetch automatically due to captcha
    // look in download directory for the file
    let download_filename = emaid_filename();
    let _metadata = match fs::metadata(download_filename.clone()) {
        Ok(m) => m,
        Err(_) => {
            // check for the exported file in Downloads
            // if it's not there, prompt to manually download
            println!("*******************************");
            println!("* ACTION REQUIRED");
            println!("*******************************");
            println!("ERC20 balances must be fetched manually because of captcha");
            println!("Open this url in your browser and download the file:");
            println!("{}", ERC20_BALANCES_URL);
            println!("Make sure the file is downloaded to");
            println!("{}", download_filename.display());
            println!("*******************************");
            std::process::exit(1);
        }
    };
    // parse emaid balances
    let mut file = fs::File::open(download_filename).unwrap();
    let mut ebody = String::new();
    file.read_to_string(&mut ebody).unwrap();
    let mut reader = csv::Reader::from_reader(ebody.as_bytes());
    let mut ebalances = Vec::<EMaidBalance>::new();
    for result in reader.deserialize() {
        let eb: EMaidBalance = result.unwrap();
        ebalances.push(eb);
    }
    ebalances
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
