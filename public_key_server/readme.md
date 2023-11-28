A service to match public keys with addresses.

Having the public key allows encrypted messages to be sent to addresses.

Many public keys can be found on the blockchain. But any address that hasn't
done an on-chain transaction will not have exposed their public key.

This accepts both bitcoin and ethereum address:pubkey pairs.

To run the server:

```
cargo run
# server starts on port 8080
```

Endpoints:

```
GET /
a webpage that allows submitting an address:pubkey pair

GET /submit?address=<addr>&pkhex=<pk>
submit an address and public key
```

Address:pubkey pairs are saved in the `pairs` directory. The filename is the
address and the file content is the public key hex.

Tests are run whenever the server is started. There's no `cargo test`.

The server should be put behind a reverse proxy such as nginx with https
enabled.
