### Summary

Tools for initial distribution from maidsafecoin to cashnotes when Safe
Network is launched.

This is intended to remove the need for a faucet service. But there may be a
need to keep the faucet for a while since emaid is not distributed and some
maidsafecoin is not able to be distributed (need a public key to distribute,
and some addresses may not have a public key available yet).

Using a snapshot of MAID balances on omni blockchain, cashnotes are sent to
those holders.

The cashnotes can't be sent to a bitcoin address because bitcoin uses ECDSA and
cashnotes use BLS, which are not compatible.

A unique private key for each cashnote is included with each distribution, and
encrypted using the bitcoin public key for each MAID address.

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

### Release and Distribute Process

1. Build the binaries

    1. `cd public_key_server; cargo build --release`

    1. `cd pubkey_submit; cargo tauri build`
        See https://tauri.app/v1/guides/distribution/publishing

    1. `cd distribute; cargo build --release`

1. Start the public key server so keys can be received from MAID holders

    1. Log in to the remote server

    1. Start the `public_key_server` service which runs on port 8080

    1. Start a reverse proxy (eg nginx) with ssl certificate for the domain
       being used (an IP is ok too but it's usually better to run it with ssl).

    1. Set the DNS for the subdomain to point to the server, eg
       `A pubkeys.mydomain.com 1.2.3.4`

    1. Any new keys submitted by users will be added to the `keys` directory

    1. Periodically make a backup of the keys so they can be used for
       distributions. Keys are stored with the address as the filename
       and the hex enconded pubkey as the content. The distribution script
       uses these files to create distributions.

1. Distribute the pubkey_submit app

    1. This is probably easiest from github. Create a new release which
       includes the binary for download. MAID holders can download the app and
       use it to submit their public key and claim distributions.

    1. Post instructions for how users can submit their public keys, explaining
       that they can't receive distributions unless their public key has been
       made available via this app or using the website hosted by the
       public_key_server.

1. Distribute funds

    1. Log in to the machine with the faucet.

    1. Ensure the faucet is running, which will also claim the tokens from
       genesis

    1. Ensure the `safe` client binary and `faucet` binary are both on `$PATH`

    1. Run the `distribute` binary on the machine with the faucet. Check the
       output to see if it ran correctly, and follow any instructions from that
       output (eg posting the distribution list to the forum).

    1. Claim the test tokens using the key in the `distribute` source code and
       the pubkey_submit app to ensure the whole process worked properly.

1. Post details to forum

    1. Users need to know how to claim their distributions.

    1. This will include instructions for a) how to submit public keys, b) how
       to fetch the distributions list c) how to claim their distribution d)
       how to get funds using the faucet if they didn't receive a distribution.
