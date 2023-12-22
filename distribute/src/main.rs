use blsttc::SecretKey;
use serde::{Deserialize, Serialize};
use std::{env, fs, process};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

const OMNI_BALANCES_URL: &str = "https://api.omniexplorer.info/ask.aspx?api=getpropertybalances&prop=3";
const CACHE_DIR: &str = "cache";
const CACHE_EXPIRY_SECS: u64 = 3600;

// TODO
// replace this hardcoded secret with distributed keygen using bls_dkg crate
const WALLET_SECRET_KEY: &str = "5920222d8798f74d09b3cdb6847e145197d52f471e42c83fe728f3dce6ce2878";

#[derive(Serialize, Deserialize)]
struct OMaidBalance {
    address: String,
    balance: String,
    reserved: String,
    public_key: Option<String>,
}

fn main() {

    run_checks();

    println!("Fetching omni balances");
    let omaid_balances = fetch_omni_balances();
    println!("Total OMaid Balances: {}", omaid_balances.len());

    let pubkey_balances = add_public_keys(omaid_balances);
    println!("Total balances with pubkeys: {}", pubkey_balances.len());

    let total_distributions_maid = total_balance(pubkey_balances);
    println!("Total to be distributed: {}", total_distributions_maid);

    println!("Fetching distribution balance from faucet");
    load_tokens_into_distribution_wallet(total_distributions_maid);

    // TODO
    // check if a wallet already exists and if so move it elsewhere
    // generate an encrypted cashnote for public keys using ECIES
    // upload cashnotes to network
    // decide what to do with addresses that have no pubkey available
}

fn run_checks() {
    // TODO
    // Check peers are available / can connect to network
    // Check the faucet is available on $PATH
    // Check the client is available on $PATH
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

fn load_tokens_into_distribution_wallet(amount_maid: u32) {
    // This uses the existing faucet functionality.
    // This doesn't use the server function of the faucet
    // because that will only issue 100 tokens at a time.
    // It uses the faucet binary
    // so must be run on the same machine as the faucet
    // with the faucet binary on $PATH.
    // TODO remove unwraps below
    let sk_vec = hex::decode(WALLET_SECRET_KEY).unwrap();
    let sk_bytes: [u8; 32] = sk_vec.as_slice().try_into().unwrap();
    let sk = SecretKey::from_bytes(sk_bytes).unwrap();
    let pk = sk.public_key();
    let pk_hex = hex::encode(pk.to_bytes());
    println!("Getting {} tokens from faucet", amount_maid);
    // the command is:
    // faucet send amount to
    let faucet_output = Command::new("faucet")
        .args(["send", &amount_maid.to_string(), &pk_hex])
        .output()
        .unwrap();
    if !faucet_output.status.success() {
        println!("FAUCET STDOUT:\n{}", String::from_utf8_lossy(&faucet_output.stdout));
        println!("FAUCET STDERR:\n{}", String::from_utf8_lossy(&faucet_output.stderr));
        panic!("Failed to get from faucet, status {}", faucet_output.status);
    }
    // print any error
    let stderr_bytes = faucet_output.stderr;
    if stderr_bytes.len() > 0 {
        println!("Faucet error:");
        println!("{}", String::from_utf8_lossy(&stderr_bytes));
    }
    // get the transfer from the output of the faucet
    let stdout_bytes = faucet_output.stdout;
    let stdout = String::from_utf8_lossy(&stdout_bytes);
    let lines = stdout.split("\n");
    let mut transfer_hex = "";
    for line in lines {
        if line.len() > 100 && hex::decode(line).is_ok() {
            transfer_hex = line;
        }
    }
    if transfer_hex.len() == 0 {
        panic!("Empty transfer from faucet");
    }
    // use our secret key for the cli wallet
    println!("Creating wallet with our sk");
    let wallet_create_output = Command::new("safe")
        .args(["wallet", "create", &WALLET_SECRET_KEY])
        .output()
        .unwrap();
    if !wallet_create_output.status.success() {
        println!("CREATE STDOUT:\n{}", String::from_utf8_lossy(&wallet_create_output.stdout));
        println!("CREATE STDERR:\n{}", String::from_utf8_lossy(&wallet_create_output.stderr));
        panic!("Failed to create wallet, status {}", wallet_create_output.status);
    }
    println!("Receiving transfer to our wallet");
    // receive the transfer using the cli
    let wallet_receive_output = Command::new("safe")
        .args(["wallet", "receive", &transfer_hex])
        .output()
        .unwrap();
    if !wallet_receive_output.status.success() {
        println!("RECEIVE STDOUT:\n{}", String::from_utf8_lossy(&wallet_receive_output.stdout));
        println!("RECEIVE STDERR:\n{}", String::from_utf8_lossy(&wallet_receive_output.stderr));
        panic!("Failed to receive transfer, status {}", wallet_receive_output.status);
    }
    println!("RECEIVE STDOUT:\n{}", String::from_utf8_lossy(&wallet_receive_output.stdout));
}

fn total_balance(balances: Vec<OMaidBalance>) -> u32 {
    let mut total_maid = 0u32;
    for b in balances {
        if b.public_key.is_some() {
            total_maid += b.balance.parse::<u32>().unwrap();
        }
    }
    total_maid
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
