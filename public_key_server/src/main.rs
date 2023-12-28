use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tide::{Response, Request};
use tide::prelude::*;
use tide_governor::GovernorMiddleware;

const KEYS_DIR: &str = "keys";

#[derive(Deserialize)]
struct AddressKey {
    address: String,
    pkhex: String,
}

#[derive(Serialize)]
struct ReqError {
    error: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tests();
    let mut app = tide::new();
    app.at("/submit")
        .with(GovernorMiddleware::per_minute(5)?)
        .get(submit);
    app.at("/")
        .with(GovernorMiddleware::per_minute(5)?)
        .serve_file("index.html")?;
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn submit(req: Request<()>) -> tide::Result {
    let qs: AddressKey = req.query()?;
    // validation
    let btc_err = validate_bitcoin_pair(&qs);
    if btc_err.len() > 0 {
        let mut res = Response::new(400);
        res.set_body(btc_err);
        return Ok(res);
    }
    // save this pair to file
    let save_err = save_to_file(&qs);
    if save_err.len() > 0 {
        let mut res = Response::new(500);
        res.set_body(save_err);
        return Ok(res);
    }
    // response
    Ok(format!("Success\nAddress: {}\nPublic Key: {}", qs.address, qs.pkhex).into())
}

fn key_filename(address: &str) -> PathBuf {
    Path::new(KEYS_DIR).join(address)
}

fn save_to_file(ak: &AddressKey) -> &str {
    // TODO consider if concurrent write is a problem here
    // I don't think there's a practical concern but it may expose a crash
    // TODO consider if the address can be manipulated into file system chaos
    let _ = fs::create_dir(KEYS_DIR);
    let filename = key_filename(&ak.address);
    let file = fs::File::create(filename);
    if !file.is_ok() {
        return "Error creating record";
    }
    let err = file.unwrap().write_all(ak.pkhex.as_bytes());
    if !err.is_ok() {
        return "Error writing record";
    }
    return "";
}

fn validate_bitcoin_pair(ak: &AddressKey) -> &str {
    // bitcoin public key is valid
    let pk = bitcoin::PublicKey::from_str(&ak.pkhex);
    if !pk.is_ok() {
        return "Invalid public key";
    }
    // bitcoin address is valid
    let addr = bitcoin::Address::from_str(&ak.address);
    if !addr.is_ok() {
        return "Invalid address";
    }
    let btc_addr = addr.clone().unwrap().require_network(bitcoin::Network::Bitcoin);
    if !btc_addr.is_ok() {
        return "Invalid network";
    }
    // bitcoin public key matches bitcoin address
    // p2pkh
    if btc_addr.clone().unwrap().is_related_to_pubkey(&pk.clone().unwrap()) {
        return "";
    }
    // p2wpkh
    let p2wpkh_addr = bitcoin::Address::p2shwpkh(&pk.clone().unwrap(), bitcoin::Network::Bitcoin).unwrap();
    if p2wpkh_addr == addr.unwrap() {
        return "";
    }
    return "Public key does not match address";
}

// Sure it's not standard to test like this but it's ok
fn tests() {
    // valid bitcoin pair
    {
        let ak = AddressKey {
            address: "1CoT3ACy3L8MUSRcRbi9FuZ8Yckz3Ghpwz".to_string(),
            pkhex: "027a41a6bef82652407562fdff7cbed487ea39e51e0010269cefcd103d421baadc".to_string(),
        };
        let err = validate_bitcoin_pair(&ak);
        assert!(err == "", "Valid bitcoin pair threw error: {}", err);
    }
    // invalid bitcoin pk (last char of pk changed)
    {
        let ak = AddressKey {
            address: "1CoT3ACy3L8MUSRcRbi9FuZ8Yckz3Ghpwz".to_string(),
            pkhex: "027a41a6bef82652407562fdff7cbed487ea39e51e0010269cefcd103d421baadd".to_string(),
        };
        let err = validate_bitcoin_pair(&ak);
        assert!(err != "", "Invalid bitcoin public key should give error but did not");
    }
    // invalid bitcoin addr
    {
        let ak = AddressKey {
            address: "invalid bitcoin address".to_string(),
            pkhex: "027a41a6bef82652407562fdff7cbed487ea39e51e0010269cefcd103d421baadc".to_string(),
        };
        let err = validate_bitcoin_pair(&ak);
        assert!(err != "", "Invalid bitcoin address should give error but did not");
    }
    // mismatched bitcoin pair
    {
        let ak = AddressKey {
            address: "1Kr6QSydW9bFQG1mXiPNNu6WpJGmUa9i1g".to_string(),
            pkhex: "027a41a6bef82652407562fdff7cbed487ea39e51e0010269cefcd103d421baadc".to_string(),
        };
        let err = validate_bitcoin_pair(&ak);
        assert!(err != "", "Mismatched bitcoin pair should give error but did not");
    }
    // p2wpkh bitcoin pair
    {
        let ak = AddressKey {
            address: "39Q6Y89u1wMYacDJw63UNiYgj3wfJtZbRj".to_string(),
            pkhex: "03608934ee3cd78469528f55bab4f1db60f3fbdd793067503dfef6d7903dbf61e9".to_string(),
        };
        let err = validate_bitcoin_pair(&ak);
        assert!(err == "", "Valid P2WPKH pair threw error: {}", err);
    }
    // junk input
    {
        let ak = AddressKey {
            address: "3".to_string(),
            pkhex: "33".to_string(),
        };
        let btc_err = validate_bitcoin_pair(&ak);
        assert!(btc_err != "", "Junk input did not return error");
    }
    // address as filename can't break stuff
    {
        let ak = AddressKey {
            address: "../../1CoT3ACy3L8MUSRcRbi9FuZ8Yckz3Ghpwz".to_string(),
            pkhex: "027a41a6bef82652407562fdff7cbed487ea39e51e0010269cefcd103d421baadc".to_string(),
        };
        let btc_err = validate_bitcoin_pair(&ak);
        assert!(btc_err != "", "Filename abuse did not return error");
    }
    // TODO
    // bitcoin pubkey mixed case
    // bitcoin compressed and uncompressed
}
