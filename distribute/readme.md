A tool to distribute initial SNT on the safe network according to the current
list of MAID holders on omni protocol blockchain.

This requires a list of bitcoin public keys gathered from the blockchain or
from submitted address/pubkey pairs from users.

The process to distribute is:

* Get the current MAID balances from omniexplorer.info

* For each balance/address

    * Create SNT cashnote for that amount with a unique secret key

    * Encrypt the cashnote + secretkey for the MAID user

    * Upload the encrypted data to Safe Network

    * Record the location of the encrypted data so the user can download it

* Publish the list of addresses + encrypted snt locations to Safe Network

To run the script:

```
cargo run
```
