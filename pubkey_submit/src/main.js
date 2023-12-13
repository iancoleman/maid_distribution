const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

let DOM = {};
DOM.online = document.querySelector(".online");
DOM.secretKey = document.querySelector(".secret-key");
DOM.secretKeyType = document.querySelector(".secret-key-type");
DOM.publicKey = document.querySelector(".public-key");
DOM.compressed = document.querySelector(".compressed");
DOM.uncompressed = document.querySelector(".uncompressed");
DOM.address = document.querySelector(".address");
DOM.submit = document.querySelector(".submit");

async function greet() {
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

function showOnline() {
    DOM.online.textContent = "You are online";
}

function showOffline() {
    DOM.online.textContent = "You are offline";
}

function trackOnlineStatus() {
    window.addEventListener('online', showOnline);
    window.addEventListener('offline', showOffline);
    if (window.navigator.onLine) {
        showOnline();
    } else {
        showOffline();
    }
}

function showCompressed() {
    DOM.secretKeyType.textContent = "compressed";
    DOM.compressed.checked = true;
}

function showUncompressed() {
    DOM.secretKeyType.textContent = "uncompressed";
    DOM.uncompressed.checked = true;
}

function getSecretKeyFromUI() {
    let skWif = DOM.secretKey.value;
    let sk = bitcoinjs.bitcoin.ECPair.fromWIF(skWif);
    return sk;
}

function secretKeyChanged(e) {
    let sk = getSecretKeyFromUI();
    if (sk.compressed) {
        showCompressed();
    }
    else {
        showUncompressed();
    }
    showPublicKey(sk);
}

function getPublicKeyFromUI() {
    let pkHex = DOM.publicKey.value;
    if (!(pkHex.length == 66 || pkHex.length == 130)) {
        console.log("Invalid pk hex");
        // TODO show this error in UI
        return
    }
    let pkBuffer = hexToBuffer(pkHex);
    if (!pkBuffer) {
        console.log("Invalid pk");
        // TODO show this error in UI
        return
    }
    let pk = bitcoinjs.bitcoin.ECPair.fromPublicKeyBuffer(pkBuffer);
    return pk;
}

function publicKeyChanged() {
    DOM.secretKey.value = "";
    let pk = getPublicKeyFromUI();
    if (pk.compressed) {
        showCompressed();
    }
    else {
        showUncompressed();
    }
    showAddress(pk);
}

function compressedChanged() {
    let compressed = DOM.compressed.checked;
    // show status in UI for sk and radios
    if (compressed) {
        showCompressed();
    }
    else {
        showUncompressed();
    }
    let keypair = getPublicKeyFromUI();
    if (DOM.secretKey.value != "") {
        keypair = getSecretKeyFromUI();
        // change the secret key in the UI
        let o = { compressed: compressed };
        let compressedSk = new bitcoinjs.bitcoin.ECPair(keypair.d, null, o);
        DOM.secretKey.value = compressedSk.toWIF();
    }
    // change the public key in the UI
    let o = { compressed: compressed };
    let pk = new bitcoinjs.bitcoin.ECPair(null, keypair.Q, o);
    showPublicKey(pk);
    // show the address
    showAddress(pk);
}

function submit() {
    // TODO
    // verify the address/pubkey
    // submit it
}

function showPublicKey(keypair) {
    let pkBuffer = keypair.getPublicKeyBuffer();
    let pkHex = bufferToHex(pkBuffer);
    DOM.publicKey.value = pkHex;
    showAddress(keypair);
}

function showAddress(pk) {
    let address = pk.getAddress();
    DOM.address.textContent = address;
}

function hexToBuffer(h) {
    if (h.length % 2 != 0) {
        return;
    }
    return bitcoinjs.Buffer.Buffer.from(h, "hex");
}

function bufferToHex(b) {
    // modified from https://stackoverflow.com/a/50868276
    return b.reduce((s, b) => s + b.toString(16).padStart(2, '0'), '');
}

function init() {
    trackOnlineStatus();
    DOM.secretKey.addEventListener("input", secretKeyChanged);
    DOM.publicKey.addEventListener("input", publicKeyChanged);
    DOM.compressed.addEventListener("change", compressedChanged);
    DOM.uncompressed.addEventListener("change", compressedChanged);
    DOM.submit.addEventListener("click", submit);
    // TODO
    // add P2WPKH addresses starting with 3
    // consider if P2SH addresses starting with 3 are needed
}

window.addEventListener("DOMContentLoaded", init);
