var myHeaders = new Headers();
myHeaders.append("x-api-key", "jdXnGbRsn0Jvt5t9");
myHeaders.append("Content-Type", "application/json");
const fs = require('fs')
const pks = JSON.parse(fs.readFileSync('/root/yet-another-searcher/client/update_pks.json', 'utf8'))
const addresses = new Set(pks)
const addressesArray = Array.from(addresses)
.filter((address) => address != "11111111111111111111111111111111")

console.log(addressesArray.length)
var requestOptions = {
    method: 'GET',
    headers: myHeaders,
    redirect: 'follow'
  };
  
  fetch("https://api.shyft.to/sol/v1/callback/list", requestOptions)
    .then(response => response.text())
    .then(result => { 
        const parsed = JSON.parse(result).result
        const callbackIds = parsed.map((callback) => callback._id)
        console.log(callbackIds)
        callbackIds.forEach((id) => {
            var raw = JSON.stringify({
                "id": id
            });

            var requestOptions = {
            method: 'DELETE',
            headers: myHeaders,
            body: raw,
            redirect: 'follow'
            };

            fetch("https://api.shyft.to/sol/v1/callback/remove", requestOptions)
            .then(response => response.text())
            .then(result => console.log(result))
            .catch(error => console.log('error', error));
        })

})
var raw = JSON.stringify({
    "network": "mainnet-beta",
    "addresses": addressesArray,
    "callback_url": "http://221.253.141.12:52116/shyft",
    "enable_events": true,
    "encoding": "RAW",
    "events": [
      "SWAP"
    ]
  });
  var requestOptions = {
    method: 'POST',
    headers: myHeaders,
    body: raw,
    redirect: 'follow'
  };
  fetch("https://api.shyft.to/sol/v1/callback/create", requestOptions)
    .then(response => response.text())
    .then(result => console.log(result))
    .catch(error => console.log('error', error));
    /*
    const createWebhook = async () => {
        try {
          const response = await fetch(
            "https://api.helius.xyz/v0/webhooks?api-key=1cc00270-904d-4624-9ee4-4f2452504cbe",
            {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                "webhookURL": "http://221.253.141.12:52116/shyft",
                                      // Wallet
                "accountAddresses": addressesArray,
                "transactionTypes": ["SWAP"],
                "encoding": "jsonParsed",
                "webhookType": "raw"
           }),
            }
          );
          const data = await response.json();
          console.log({ data });
        } catch (e) {
          console.error("error", e);
        }
      };
      createWebhook();*/