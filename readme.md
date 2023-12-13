### Summary

Tools for initial distribution of cashnotes when Safe Network is launched.

This removes the need for a faucet service.

Using a snapshot of MAID balances on omni blockchain, we can send cashnotes to
those holders.

The cashnotes can't be sent to a bitcoin address because bitcoin uses ECDSA and
SNT uses BLS, which are not compatible.

The private key for the SNT is included with cashnote, and encrypted using the
bitcoin public key for each MAID address.

ERC20 eMaid will continue to run so is not included in this distribution.

### Tools

**pubkey_submit**

A gui to allow MAID users to easily and securely submit the public key to
maidsafe so cashnotes can be encrypted to their MAID address.

**public_key_server**

A service to be run by maidsafe to collect public keys that match bitcoin
addresses.

Many bitcoin public keys are already available on the blockchain, but some are
not so this service allows those keys to be made available to maidsafe.

**distribute**

A script to be run by maidsafe that will distribute cashnotes when a network
is launched (testnet or mainnet).

It uses the list of addresses and publickeys from the `pubkey_submit` and
`public_key_server` tools.
