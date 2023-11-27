use sha3::{Digest, Keccak256};
use std::str::FromStr;
use tide::{Response, Request};
use tide::prelude::*;

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
    app.at("/submit").get(submit);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn submit(req: Request<()>) -> tide::Result {
    let qs: AddressKey = req.query()?;
    // validation
    let btc_err = validate_bitcoin_pair(&qs);
    let eth_err = validate_ethereum_pair(&qs);
    if btc_err.len() > 0 && eth_err.len() > 0 {
        let mut res = Response::new(400);
        res.set_body(btc_err);
        return Ok(res);
    }
    // TODO save this pair to file
    // response
    Ok(format!("Success\nAddress: {}\nPublic Key: {}", qs.address, qs.pkhex).into())
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
    let btc_addr = addr.unwrap().require_network(bitcoin::Network::Bitcoin);
    if !btc_addr.is_ok() {
        return "Invalid network";
    }
    // bitcoin public key matches bitcoin address
    if !btc_addr.unwrap().is_related_to_pubkey(&pk.unwrap()) {
        return "Public key does not match address";
    }
    return "";
}

fn validate_ethereum_pair(ak: &AddressKey) -> &str {
    // ethereum address is valid
    let eth_addr = alloy_primitives::Address::parse_checksummed(&ak.address, None);
    if !eth_addr.is_ok() {
        return "Invalid address";
    }
    // ethereum public key matches ethereum address
    // converting the pk to address also checks the pk is valid
    let pk_addr = eth_pk_to_addr(&ak.pkhex);
    if pk_addr == "" {
        return "Invalid public key";
    }
    let addr_no_checksum = ak.address.to_lowercase();
    if pk_addr != addr_no_checksum {
        return "Public key does not match address";
    }
    // TODO decide if checksum should be valid or not; after all a matching pk
    // is still a match regardless if the checksum is present or not.
    return "";
}

// https://docs.rs/ethereum-private-key-to-address/latest/src/ethereum_private_key_to_address/lib.rs.html#82-92
fn eth_pk_to_addr(pk: &String) -> String {
    // remove 0x prefix if it's there
    if pk.len() < 2 {
        return "".to_string();
    }
    let mut pk_no_prefix: String = pk.clone();
    let prefix: String = pk.chars().take(2).collect();
    if prefix == "0x" {
        pk_no_prefix = pk.chars().skip(2).take(pk.len()-2).collect();
    }
    // convert hex to a secp256k1 public key
    let pkbytes = hex::decode(pk_no_prefix);
    if !pkbytes.is_ok() {
        return "".to_string();
    }
    let pksecp = secp256k1::PublicKey::from_slice(&pkbytes.unwrap());
    if !pksecp.is_ok() {
        return "".to_string();
    }
    let public_key = pksecp.unwrap().serialize_uncompressed()[1..].to_vec();
    // convert to eth address
    let mut hasher = Keccak256::new();
    hasher.update(public_key);
    let address = hasher.finalize();
    let mut addr = hex::encode(&address[12..32]);
    addr.insert_str(0, "0x");
    addr
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
    // valid ethereum pair
    {
        let ak = AddressKey {
            address: "0x02232c97129ab3d6fFD59192550be13a09e3be9a".to_string(),
            pkhex: "0x0329a4178ecb95f18b0eccf6ecd8fc28416503072de31f0852ba43abe26839995b".to_string(),
        };
        let err = validate_ethereum_pair(&ak);
        assert!(err == "", "Valid ethereum pair threw error: {}", err);
    }
    // invalid ethereum address
    {
        let ak = AddressKey {
            address: "invalid ethereum address".to_string(),
            pkhex: "0x0329a4178ecb95f18b0eccf6ecd8fc28416503072de31f0852ba43abe26839995b".to_string(),
        };
        let err = validate_ethereum_pair(&ak);
        assert!(err != "", "Invalid ethereum address should give error but did not");
    }
    // invalid ethereum pk (last char of public key changed)
    {
        let ak = AddressKey {
            address: "0x02232c97129ab3d6fFD59192550be13a09e3be9a".to_string(),
            pkhex: "0x0329a4178ecb95f18b0eccf6ecd8fc28416503072de31f0852ba43abe26839995c".to_string(),
        };
        let err = validate_ethereum_pair(&ak);
        assert!(err != "", "Invalid ethereum public key should give error but did not");
    }
    // ethereum pair that doesn't match
    {
        let ak = AddressKey {
            address: "0x02232c97129ab3d6fFD59192550be13a09e3be9a".to_string(),
            pkhex: "0x02f70bce59894a77796f0da88d8451d7fc3341460bbf09fcf3383b22b0a6c83272".to_string(),
        };
        let err = validate_ethereum_pair(&ak);
        assert!(err != "", "Mismatched ethereum pair should give error but did not");
    }
    // junk input
    {
        let ak = AddressKey {
            address: "3".to_string(),
            pkhex: "33".to_string(),
        };
        let btc_err = validate_bitcoin_pair(&ak);
        assert!(btc_err != "", "Junk input did not return error");
        let eth_err = validate_ethereum_pair(&ak);
        assert!(eth_err != "", "Junk input did not return error");
    }
    // TODO
    // bitcoin pubkey mixed case
    // bitcoin compressed and uncompressed
    // bitcoin addr starting with 3
    // eth compressed and uncompressed
    // eth address with btc pubkey
    // btc address with eth pubkey
    // eth with checksum
    // eth with no hex prefix for both addr and/or pk
}
