import * as token from "@solana/spl-token"
import * as web3 from "@solana/web3.js";
import { Buffer } from 'buffer';

import BN from 'bn.js';

const fs = require('fs');

async function read_dir_names(dir_name: string) {
    const dir = fs.opendirSync(dir_name)
    let dirent
    let dirs = []
    while ((dirent = dir.readSync()) !== null) {
        dirs.push(dirent.name)
    }
    dir.closeSync()
    return dirs
}


function chunk(array, chunkSize) {
    var R = [];
    for (var i = 0; i < array.length; i += chunkSize) {
        R.push(array.slice(i, i + chunkSize));
    }
    return R;
}

import { publicKey, struct, u32, u64, u8, option, vec } from '@project-serum/borsh';

// l m f a o 
//* AccountLayout.encode from "@solana/spl-token" doesn't work : https://github.com/solana-labs/solana-program-library/blob/27a58282d354ab3596e48a9be7957eec511b8b8e/stake-pool/js/src/layouts.ts#L14
export const AccountLayout = struct<token.AccountInfo>([
    publicKey('mint'),
    publicKey('owner'),
    u64('amount'),
    u32('delegateOption'),
    publicKey('delegate'),
    u8('state'),
    u32('isNativeOption'),
    u64('isNative'),
    u64('delegatedAmount'),
    u32('closeAuthorityOption'),
    publicKey('closeAuthority'),
]);

async function main() {    
    
    let connection = new web3.Connection("https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9");
    let programs = [];
    let accounts = [];
    let mints = [];

    // get all the accounts to clone

    // ORCA POOL SETUP 
    let orca_pid = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
    programs.push(orca_pid) 

    let orca_pools = "../pools/orca/";
    let orca_pool_names: string[] = fs.readdirSync(orca_pools);
    var orca_count = 0;
    orca_pool_names.forEach(name => {
        let pool = JSON.parse(fs.readFileSync(orca_pools+name));
        orca_count += 1 
        pool.tokenIds.forEach(mintId => {
            accounts.push(pool.tokens[mintId].addr)
            mints.push(mintId)
        });
        accounts.push(pool.address);
        accounts.push(pool.poolTokenMint);
        accounts.push(pool.feeAccount);    
    });
    let prog_accs = await connection.getProgramAccounts(new web3.PublicKey(orca_pid))
    prog_accs.forEach(v => {
        accounts.push(v.pubkey.toString())
        orca_count += 1
    })
    console.log("orca count", orca_count)


    // SABER POOL SETUP 
    let saber_pid = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
    programs.push(saber_pid) 
    // read testing pool 
    let saber_pools = "../pools/saber/";
    let saber_pool_names = fs.readdirSync(saber_pools);
    console.log(saber_pool_names)
    let saber_count = 0; 
    for (var name of saber_pool_names) {
        let pool = JSON.parse(fs.readFileSync(saber_pools+"/"+name));
        saber_count += 1;
        // console.log(pool)
        accounts.push(pool.pool_account)
        accounts.push(pool.pool_token_mint)
        for (var item of Object.keys(pool.addresses)) {
            console.log(item)
            item = pool.addresses[item]
            console.log(item)
            try {
                accounts.push(new web3.PublicKey(item).toBase58())
            }
            catch (err){
                console.log(err)
                accounts.push(new web3.PublicKey(item[0]).toBase58())
                accounts.push(new web3.PublicKey(item[1]).toBase58())

            }
        }
        for (var mint of pool.underlyingTokens){
            mints.push(mint)
        }
    }
    console.log("saber_count", saber_count)

    // ALDRIN POOL 
    let aldrin_v1 = "AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6"
    programs.push(aldrin_v1)
    let aldrin_v2 = "CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4"
    programs.push(aldrin_v2)
    // read testing pool 
    let aldrin_pools = "../pools/aldrin/";
    let aldrin_pool_names = fs.readdirSync(aldrin_pools);
    let aldrin_count = 0; 
    for (var name of aldrin_pool_names) {
        let pool = JSON.parse(fs.readFileSync(aldrin_pools+name));
        // console.log(pool)
        aldrin_count += 1;
        accounts.push(pool.poolPublicKey)
        accounts.push(pool.poolMint)
        // fees 
        accounts.push(pool.feePoolTokenAccount)
        accounts.push(pool.feeBaseAccount)
        accounts.push(pool.feeQuoteAccount)
        accounts.push(pool.lpTokenFreezeVault)
        if (pool.poolVersion == 2) {
            accounts.push(pool.curve);
        }
        accounts.push(pool.baseTokenVault)
        accounts.push(pool.quoteTokenVault)
        mints.push(pool.baseTokenMint)
        mints.push(pool.quoteTokenMint)
    }
    console.log("aldrin_count", aldrin_count)

    // SERUM AMM 
    let serum_pid = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
    programs.push(serum_pid) 

    let serum_pools = "../pools/openbook/"; 
    let serum_pool_names = fs.readdirSync(serum_pools);
    let serum_count = 0; 
    for (let name of serum_pool_names) {
        let pool = JSON.parse(fs.readFileSync(serum_pools+name));
        serum_count += 1;
    
        accounts.push(pool.ownAddress);
        accounts.push(pool.requestQueue);    
        accounts.push(pool.bids);
        accounts.push(pool.asks);
        accounts.push(pool.baseVault);
        accounts.push(pool.quoteVault);
        accounts.push(pool.eventQueue);
    
        // accounts.push(pool.vaultSigner);
        // let vaultSigner = (await web3.PublicKey.findProgramAddress(
        //     [new web3.PublicKey(pool.ownAddress).toBuffer()],
        //     new web3.PublicKey(serum_pid)
        // ))[0];
        // accounts.push(vaultSigner);
        
        mints.push(pool.baseMint);
        mints.push(pool.quoteMint);
    }
    console.log("serum_count", serum_count)

    // fee-based not swapable
    let other = "MSRMcoVyrFxnSgo5uXwone5SKcGhT1KEJMFEkMEWf9L"
    accounts.push(other)

    // // JUPITER SWAP PROGRAM 
    // let jupiter_addr = "JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo"
    // programs.push(jupiter_addr)
    
    //  my mainnet ATAs
    // let pk_str = "GmhAnaL9UkY23zxbfzpt7UPYM5dcdN3iGXLCFD7RtBrE";
    // let my_pubkey = new web3.PublicKey(pk_str);
    // let token_accounts_resp = await connection.getTokenAccountsByOwner(my_pubkey, {
    //     programId: token.TOKEN_PROGRAM_ID,
    // });
    // // we will modify these explicitly 
    // let ata_accounts = [];
    // token_accounts_resp.value.forEach(v => {
    //     ata_accounts.push(v.pubkey.toString())
    // });
    // accounts.push(pk_str)

    // remove duplicates 
    accounts = [...new Set(accounts)];
    programs = [...new Set(programs)];
    mints = [...new Set(mints)];

    console.log("saving programs:", programs.length)
    console.log("saving accounts:", accounts.length)
    console.log("mints:", mints.length);

    // save mints 
    fs.writeFile(`saved_mints.json`, JSON.stringify(mints), 'utf8', ()=>{});

    // new owner pk so we dont fck the mainnet acc up in testing lol 
    let owner = web3.Keypair.generate();
    console.log('owner pK:', owner.publicKey.toString());
    // save secret 
    fs.writeFile(`localnet_owner.key`, "["+owner.secretKey.toString()+"]", () => {});  

    // equivalent to 
    // command = "solana account -u m \
    // --output json-compact \
    // --output-file accounts/{}.json {}" 
    // BATCHED >:)

    console.log("scrapping accounts...")
    let n = 0;
    for (let acc_chunk of chunk(accounts, 99)) {
        console.log(99 * n, "/", accounts.length);
        n += 1;
        let accounties = acc_chunk.map((s) => {
            try {
            
            return new web3.PublicKey(s)
            }
            catch (err){
                console.log(err)
                return null
            }
        })
        // remove nulls
        accounties = accounties.filter(function (el) {
            return el != null;
        })
        let infos = await connection.getMultipleAccountsInfo(accounties)
        
        for (let i=0; i < infos.length; i ++) {
            let pk = acc_chunk[i];
            var info = infos[i]
            
            let notype_info = JSON.parse(JSON.stringify(info, null, "\t"))
            if (!info){
                continue
            }
            notype_info.data = [info.data.toString("base64"), "base64"]

            let local_validator_acc = {
                account: notype_info, 
                pubkey: pk,
            }
            let path = `accounts/${pk}.json`; 
            fs.writeFile(path, JSON.stringify(local_validator_acc), 'utf8', ()=>{});
        }
    }

    fs.writeFile(`programs.json`, JSON.stringify(programs, null, "\t"), 'utf8', ()=>{});

    console.log("scrapping mints...")
    for (let acc_chunk of chunk(mints, 99)) {
        let infos = await connection.getMultipleAccountsInfo(acc_chunk.map(s => new web3.PublicKey(s)))
        
        for (let i=0; i < infos.length; i ++) {
            let pk = acc_chunk[i];
            var info = infos[i]
            // let data = info.data; 

            let mint = token.MintLayout.decode(info.data);
            mint.mintAuthority = owner.publicKey.toBuffer(); // allow me to mint more :)
            mint.mintAuthorityOption = 1; // :)
            let data = Buffer.alloc(token.MintLayout.span);
            token.MintLayout.encode(mint, data);

            let notype_info = JSON.parse(JSON.stringify(info, null, "\t"))
            notype_info.data = [data.toString("base64"), "base64"] // !! 
            let local_validator_acc = {
                account: notype_info, 
                pubkey: pk,
            }
            let path = `accounts/${pk}.json`; 
            fs.writeFile(path, JSON.stringify(local_validator_acc), 'utf8', ()=>{});
        }
    }

}

main()