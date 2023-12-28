### yet another searcher

#### download and build
```
$ git clone git@github.com:staccDOTsol/yet-another-searcher.git --recursive
$ cd client/
$ cargo build
```

#### run
```
$ cargo run --bin main -- --cluster mainnet --keypair <keypair> --flashing no
```

#### test
```
$ avm use 0.22.1
$ cargo test # to run test the spot quotes with mainnet forked local validator
```

#### directory info
```
client/: searcher client
onchain-data/:

```