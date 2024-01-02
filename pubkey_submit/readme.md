A simple user interface to allow MAID users to submit their address and pubkey
to the `public_key_server` service.

To run it

```
cargo tauri dev
```

For production:

* Change `const submitUrl` in `main.js` to the correct remote server address

* Change `allowlist.http.scope` in `tauri.conf.json` to the remote server
