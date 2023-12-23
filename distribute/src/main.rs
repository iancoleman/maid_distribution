use blsttc::SecretKey;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::time::SystemTime;

const OMNI_BALANCES_URL: &str = "https://api.omniexplorer.info/ask.aspx?api=getpropertybalances&prop=3";
const CACHE_DIR: &str = "cache";
const CACHE_EXPIRY_SECS: u64 = 3600;

// TODO
// replace this hardcoded secret with distributed keygen using bls_dkg crate
const WALLET_SECRET_KEY: &str = "5920222d8798f74d09b3cdb6847e145197d52f471e42c83fe728f3dce6ce2878";
const ENCRYPTED_MD_DIR: &str = "encrypted_maid_distributions";

// Generated with bip39 phrase
// wedding pig fiscal
// bip44 derivation path m/44'/0'/0'/0/0
const TEST_BITCOIN_ADDRESS: &str = "1LyVLuxCbgLgYCZ6Sk6BrPJqAhixuyJpP7";
const TEST_BITCOIN_PUBLIC_KEY: &str = "02888b3476298033f5f6ac52f868d603ace34de8918944a2ecde9b61e751132926";
const _TEST_BITCOIN_SECRET_KEY: &str = "KyNdvxT1Ead7AD9thdvg8399fVxC1Tdf9FvPc2dqmmnHstcTUH5y";

#[derive(Serialize, Deserialize)]
struct OMaidBalance {
    address: String,
    balance: String,
    reserved: String,
    public_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MaidDistribution {
    #[serde(with = "serde_bytes")]
    transfer: Vec<u8>,
    #[serde(with = "serde_bytes")]
    secret_key: Vec<u8>,
}

fn main() {

    run_checks();

    println!("Fetching omni balances");
    let omaid_balances = fetch_omni_balances();
    println!("Total OMaid Balances: {}", omaid_balances.len());

    let mut pubkey_balances = add_public_keys(&omaid_balances);
    println!("Total balances with pubkeys: {}", pubkey_balances.len());

    // Add an extra entry for the test address.
    // In production this may be excluded.
    pubkey_balances.push(OMaidBalance{
        address: TEST_BITCOIN_ADDRESS.to_string(),
        balance: "1".to_string(),
        reserved: "0".to_string(),
        public_key: Some(TEST_BITCOIN_PUBLIC_KEY.to_string()),
    });

    let distribution_amount = total_balance(&pubkey_balances);
    // Need a little extra in the wallet to upload the data to the safe network
    let upload_amount = 1;
    let total_distributions_maid = distribution_amount + upload_amount;
    println!("Total to be distributed: {}", total_distributions_maid);

    println!("Fetching distribution balance from faucet");
    load_tokens_into_distribution_wallet(total_distributions_maid);

    println!("Creating distributions");
    distribute_tokens(&pubkey_balances);
}

fn run_checks() {
    // TODO
    // check if a wallet already exists and if so move it elsewhere
    // Check peers are available / can connect to network
    // Check the faucet is available on $PATH
    // Check the client is available on $PATH
    // create encrypted md directory if not exist
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

fn add_public_keys(balances: &Vec<OMaidBalance>) -> Vec<OMaidBalance> {
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
            address: balance.address.clone(),
            balance: balance.balance.clone(),
            reserved: balance.reserved.clone(),
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

fn maid_distribution_filepath(maid_address: String) -> PathBuf {
    Path::new(ENCRYPTED_MD_DIR).join(maid_address)
}

fn distribute_tokens(balances: &Vec<OMaidBalance>) {
    let mut all_encrypted_maid_distributions_csv = "MAID address,Distribution\n".to_string();
    for b in balances {
        // check it has a public key
        if b.public_key.is_none() {
            continue;
        }
        // check it has a valid balance
        let balance = b.balance.parse::<u32>().unwrap();
        if balance == 0 {
            continue;
        }
        // check if this has already been distributed
        let _ = fs::create_dir(ENCRYPTED_MD_DIR);
        let md_filepath = maid_distribution_filepath(b.address.clone());
        let mut encrypted_md_hex = String::new();
        if md_filepath.exists() {
            // read existing MaidDistribution from file
            let mut file = fs::File::open(md_filepath.clone()).unwrap();
            file.read_to_string(&mut encrypted_md_hex).unwrap();
        }
        else {
            // create new encrypted MaidDistribution for this maid address
            println!("Creating distribution of {} tokens for {}", b.balance.clone(), b.address.clone());
            encrypted_md_hex = create_new_maid_distribution(b);
        }
        // keep track of the upload location and the maid address
        let row = format!("{},{}\n", b.address, encrypted_md_hex);
        all_encrypted_maid_distributions_csv += &row;
    }
    // save all_encrypted_maid_distributions_csv
    let csv_filepath = maid_distribution_filepath("all_distributions.csv".to_string());
    let mut file = fs::File::create(csv_filepath.clone()).unwrap();
    let _ = file.write_all(&all_encrypted_maid_distributions_csv.as_bytes());
    // upload the list of addresses -> encrypted MaidDistribution
    let upload_output = Command::new("safe")
        .args(["files", "upload", &csv_filepath.as_os_str().to_str().unwrap()])
        .output()
        .unwrap();
    if !upload_output.status.success() {
        println!("UPLOAD STDOUT:\n{}", String::from_utf8_lossy(&upload_output.stdout));
        println!("UPLOAD STDERR:\n{}", String::from_utf8_lossy(&upload_output.stderr));
        panic!("Failed to upload MaidDistribution, status {}", upload_output.status);
    }
    let upload_stdout_bytes = upload_output.stdout;
    let upload_stdout = String::from_utf8_lossy(&upload_stdout_bytes);
    // get the address of the uploaded data
    let words = upload_stdout.split_whitespace();
    let mut csv_address = "";
    for word in words {
        if word.len() == 64 && hex::decode(word).is_ok() {
            csv_address = word;
        }
    }
    if csv_address.len() == 0 {
        panic!("No address for uploaded MaidDistribution list");
    }
    // print out the location of that mapping
    println!("Address for distribution csv: {}", csv_address);
}

fn create_new_maid_distribution(b: &OMaidBalance) -> String {
    // Generate random key for the maid user to use for spending.
    // This key should be generated from dkg, so this step will change in
    // the future.
    // For testnets, so long as the recipient key is never stored or known
    // the process is safe enough.
    let recipient_sk = SecretKey::random();
    let recipient_pk = recipient_sk.public_key();
    let recipient_pk_hex = hex::encode(recipient_pk.to_bytes());
    // generate a transfer to this public key
    let wallet_send_output = Command::new("safe")
        .args(["wallet", "send", &b.balance, &recipient_pk_hex])
        .output()
        .unwrap();
    if !wallet_send_output.status.success() {
        println!("SEND STDOUT:\n{}", String::from_utf8_lossy(&wallet_send_output.stdout));
        println!("SEND STDERR:\n{}", String::from_utf8_lossy(&wallet_send_output.stderr));
        panic!("Failed to send transfer, status {}", wallet_send_output.status);
    }
    let stdout_bytes = wallet_send_output.stdout;
    let stdout = String::from_utf8_lossy(&stdout_bytes);
    let lines = stdout.split("\n");
    let mut transfer_hex = "";
    for line in lines {
        if line.len() > 100 && hex::decode(line).is_ok() {
            transfer_hex = line;
        }
    }
    if transfer_hex.len() == 0 {
        panic!("Empty transfer to {}", b.address);
    }
    let transfer_bytes = hex::decode(transfer_hex).unwrap();
    // create a MaidDistribution using this information
    let md = MaidDistribution{
        transfer: transfer_bytes.clone(),
        secret_key: recipient_sk.to_bytes().to_vec(),
    };
    // encode the MD using messagepack
    let md_bytes = rmp_serde::to_vec(&md).unwrap();
    // encrypt the messagepack bytes using ECIES and bitcoin public key
    let maid_pk_hex = b.public_key.clone().unwrap();
    let maid_pk_bytes = hex::decode(maid_pk_hex).unwrap();
    let encrypted_md = ecies::encrypt(&maid_pk_bytes, &md_bytes).unwrap();
    let encrypted_md_hex = hex::encode(encrypted_md);
    // save encrypted md to file
    let md_filepath = maid_distribution_filepath(b.address.clone());
    let mut file = fs::File::create(md_filepath.clone()).unwrap();
    let _ = file.write_all(&encrypted_md_hex.as_bytes());
    encrypted_md_hex
}

fn total_balance(balances: &Vec<OMaidBalance>) -> u32 {
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
