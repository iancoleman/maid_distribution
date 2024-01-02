const invoke = window.__TAURI__.tauri.invoke;
const fetch = window.__TAURI__.http.fetch;
const ResponseType = window.__TAURI__.http.ResponseType;

const submitUrl = "http://127.0.0.1:8080/submit";

let greetInputEl;
let greetMsgEl;

let DOM = {};
DOM.online = document.querySelectorAll(".online");
DOM.secretKey = document.querySelector(".secret-key");
DOM.secretKeyType = document.querySelector(".secret-key-type");
DOM.publicKey = document.querySelector(".public-key");
DOM.compressed = document.querySelector(".compressed");
DOM.uncompressed = document.querySelector(".uncompressed");
DOM.p2wpkh = document.querySelector(".p2wpkh");
DOM.address = document.querySelector(".address");
DOM.submit = document.querySelector(".submit");
DOM.submitFeedback = document.querySelector(".feedback");
DOM.clear = document.querySelectorAll(".clear-secrets");

const ONLINE_STR = "This computer is currently online and connected to the internet";
const OFFLINE_STR = "This computer is currently offline";

async function greet() {
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

function showOnline() {
    for (let i=0; i<DOM.online.length; i++) {
        DOM.online[i].textContent = ONLINE_STR;
    }
}

function showOffline() {
    for (let i=0; i<DOM.online.length; i++) {
        DOM.online[i].textContent = OFFLINE_STR;
    }
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

function showP2wpkh() {
    DOM.secretKeyType.textContent = "p2wpkh";
    DOM.p2wpkh.checked = true;
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
    else if (sk) {
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

function pkTypeChanged() {
    let compressed = DOM.compressed.checked || DOM.p2wpkh.checked;
    // show status in UI for sk and radios
    if (DOM.compressed.checked) {
        showCompressed();
    }
    else if (DOM.uncompressed.checked) {
        showUncompressed();
    }
    else if (DOM.p2wpkh.checked) {
        showP2wpkh();
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
    let address = DOM.address.textContent.trim();
    let pkhex = DOM.publicKey.value.trim();
    if (pkhex.length == 0) {
        DOM.submitFeedback.textContent = "Empty public key";
        return;
    }
    if (address.length == 0) {
        DOM.submitFeedback.textContent = "Empty address";
        return;
    }
    DOM.submitFeedback.textContent = "Submitting...";
    let params = "?address=" + address + "&pkhex=" + pkhex;
    let url = submitUrl + params;

    fetch(url, {
      method: "GET",
      timeout: 30, //seconds
      responseType: ResponseType.Text,
    })
      .then((resp) => {
        if (resp.ok) {
            DOM.submitFeedback.textContent = "Public key submitted";
        }
        else {
            let errMsg = "Error submitting: " + resp.data;
            DOM.submitFeedback.textContent = errMsg;
        }
      })
      .catch((e) => {
          let errMsg = "Error: " + e;
          DOM.submitFeedback.textContent = errMsg;
      });
}

function showPublicKey(keypair) {
    let pkBuffer = keypair.getPublicKeyBuffer();
    let pkHex = bufferToHex(pkBuffer);
    DOM.publicKey.value = pkHex;
    showAddress(keypair);
}

function showAddress(pk) {
    let address = pk.getAddress();
    if (DOM.p2wpkh.checked) {
        let keyhash = bitcoinjs.bitcoin.crypto.hash160(pk.getPublicKeyBuffer());
        let scriptsig = bitcoinjs.bitcoin.script.witnessPubKeyHash.output.encode(keyhash);
        let addressbytes = bitcoinjs.bitcoin.crypto.hash160(scriptsig);
        let scriptpubkey = bitcoinjs.bitcoin.script.scriptHash.output.encode(addressbytes);
        let network = bitcoinjs.bitcoin.networks.bitcoin;
        address = bitcoinjs.bitcoin.address.fromOutputScript(scriptpubkey, network);
    }
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

function clearSecrets() {
    DOM.secretKey.value = "";
    // TODO consider clearing clipboard if it contains a secret key?
}

function init() {
    trackOnlineStatus();
    DOM.secretKey.addEventListener("input", secretKeyChanged);
    DOM.publicKey.addEventListener("input", publicKeyChanged);
    DOM.compressed.addEventListener("change", pkTypeChanged);
    DOM.uncompressed.addEventListener("change", pkTypeChanged);
    DOM.p2wpkh.addEventListener("change", pkTypeChanged);
    DOM.submit.addEventListener("click", submit);
    DOM.clear.forEach((e) => {
        e.addEventListener("click", clearSecrets);
    });
}

window.addEventListener("DOMContentLoaded", init);
