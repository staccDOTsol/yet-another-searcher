// @ts-nocheck
import { Prism } from "@prism-hq/prism-ag";
import { Connection, PublicKey } from "@solana/web3.js";

import {getAssociatedTokenAddress} from "@project-serum/associated-token";
import fs from 'fs'
import { StableSwap } from "@saberhq/stableswap-sdk";
import {
  deserializeAccount,
  deserializeMint,
  parseBigintIsh,
  Token,
  TokenAmount,
} from "@saberhq/token-utils";
export interface LiqProviders {
    serum?: boolean;
    saber?: boolean;
    raydium?: boolean;
    orca?: boolean;
    openbook?: boolean;

    aldrin?: boolean;
    lifinity?: boolean;
    lifinityV2?: boolean;
    symmetry?: boolean;
    cropper?: boolean;
    sencha?: boolean;
    saros?: boolean;
    step?: boolean;
    penguin?: boolean;
    mercurial?: boolean;
    stepn?: boolean;
    marinade?: boolean;
    cykura?: boolean;
    gooseFX?: boolean;
    balansol?: boolean;
    whirlpools?: boolean;
    raydiumClmm?: boolean;
    marcopolo?: boolean;
    phoenix?: boolean;
}
function safeStringify(obj: any, indent = 2) {
    let cache: any[] = [];
    const retVal = JSON.stringify(
      obj,
      (key, value) =>
        typeof value === "object" && value !== null
          ? cache.includes(value)
            ? undefined // Duplicate reference found, discard key
            : cache.push(value) && value // Store value in our collection
          : value,
      indent
    );
    cache = null!;
    return retVal;
}
async function main(){
    const connection = new Connection("https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718") ;
let prism = await Prism.init({
    // user executing swap
    connection,
    liqProviders: {"openbook": true, "serum": true, "orca": true, "raydium": true, "saber": true,
    "aldrin": false,
    "lifinity": false,
    "lifinityV2": false,
    "symmetry": false,
    "cropper": false,
    "sencha": false,
    "saros": false,
    "step": false,
    "penguin": false,
    "mercurial": false,
    "stepn": false,
    "marinade": false,
    "cykura": false,
    "gooseFX": false,
    "balansol": false,
    "whirlpools": false,
    "raydiumClmm": false,
    "marcopolo": false,
    "phoenix": false,
}
});
function transformRaydium(data: any): any {
  console.log(data)
    const transformedData = {
      id: data.ammId.toBase58(),
      ammOwner : data.ammOwner.toBase58(),
      baseMint: data.coinMintAddress.toBase58(),
      quoteMint: data.pcMintAddress.toBase58(),
      lpMint: data.lpMintAddress.toBase58(),
      baseDecimals: data.coinDecimals.toNumber(),
      quoteDecimals: data.pcDecimals.toNumber(),
      lpDecimals: data.coinDecimals.toNumber(), // Assuming lpDecimals is the same as coinDecimals
      version: 4, // Assuming version is a constant
      programId: data.serumProgramId.toBase58(),
      openOrders: data.ammOpenOrders.toBase58(),
      targetOrders: data.ammTargetOrders.toBase58(),
      baseVault: data.poolCoinTokenAccount.toBase58(),
      quoteVault: data.poolPcTokenAccount.toBase58(),
      withdrawQueue: data.poolWithdrawQueue.toBase58(),
      lpVault: data.poolTempLpTokenAccount.toBase58(),
      marketVersion: 3, // Assuming marketVersion is a constant
      marketProgramId: data.serumProgramId.toBase58(),
      marketId: data.serumMarket.toBase58(),
      marketAuthority: data.ammOwner.toBase58(),
      marketBaseVault: data.poolCoinTokenAccount.toBase58(),
      marketQuoteVault: data.poolPcTokenAccount.toBase58(),
      marketBids: data.ammOpenOrders.toBase58(),
      marketAsks: data.ammTargetOrders.toBase58(),
      marketEventQueue: data.poolWithdrawQueue.toBase58(),
      lookupTableAccount: data.poolTempLpTokenAccount.toBase58()
    };
  
    return transformedData;
  }
 async function transformSaber(input) {
    let tokens = {}
    let fee_accounts = {}
    const stableSwap = await StableSwap.load(connection, new PublicKey(input.addresses.swapAccount));

    for (var tokenId in input.displayTokens){
        let ata = await getAssociatedTokenAddress(new PublicKey("PoNA1qzqHWar3g8Hy9cxA2Ubi3hV7q84dtXAxD77CSD"), new PublicKey(input.displayTokens[tokenId]))
        let fee_ata = stableSwap.state.tokenA.mint.toBase58() == input.underlyingTokens[tokenId] ? stableSwap.state.tokenA.feeAccount : stableSwap.state.tokenB.feeAccount
        
        fee_accounts[input.displayTokens[tokenId]] = fee_ata
            let token = {
                tag: "heh",
                name: "heh",
                mint: input.displayTokens[tokenId],
                scale: 0,
                addr: ata.toBase58(),
            }
          tokens[input.displayTokens[tokenId]] = token
    }
    return {
        poolAccount: input.addresses.swapAccount,
        authority: input.addresses.admin,
        poolTokenMint: input.addresses.lpTokenMint,
        tokenIds: input.underlyingTokens,
        feeAccounts: fee_accounts,
        tokens: tokens, // You need to provide a way to get this value
        targetAmp: 0, // You need to provide a way to get this value
        feeNumerator: 0, // You need to provide a way to get this value
        feeDenominator: 1000, // You need to provide a way to get this value
        feeAccounts: fee_accounts, // You need to provide a way to get this value
      };
    
  }
async function transformData(orca) {
    const poolParams = orca.pool.poolParams;
    let tokens = {};
for (var tokenId in poolParams.tokenIds){
    let ata = await getAssociatedTokenAddress(new PublicKey("PoNA1qzqHWar3g8Hy9cxA2Ubi3hV7q84dtXAxD77CSD"), new PublicKey(poolParams.tokenIds[tokenId]))

        let token = {
            tag: "heh",
            name: "heh",
            mint: poolParams.tokenIds[tokenId],
            scale: 0,
            addr: ata.toBase58(),
        }
      tokens[poolParams.tokenIds[tokenId]] = token
}
    return {
      address: poolParams.address.toBase58(),
      nonce: poolParams.nonce,
      authority: poolParams.authority.toBase58(),
      poolTokenMint: poolParams.poolTokenMint.toBase58(),
      poolTokenDecimals: poolParams.poolTokenDecimals,
      feeAccount: poolParams.feeAccount.toBase58(),
      tokenIds: poolParams.tokenIds,
      tokens: tokens,
      curveType: poolParams.curveType,
      feeStructure: {
        traderFee: {
            numerator: poolParams.feeStructure.traderFee.numerator.toNumber(),
            denominator: poolParams.feeStructure.traderFee.denominator.toNumber(),
        },
        ownerFee: {
            numerator: poolParams.feeStructure.ownerFee.numerator.toNumber(),
            denominator: poolParams.feeStructure.ownerFee.denominator.toNumber(),
        },
        },
      amp: 0
    };
  }
  
const lps = prism.liquidityProviders;
for (var lp of Object.values(lps)){
    lp = Object.values(lp)[0]
    const provider = lp.provider;
    if (provider == "saber"){

        const toWrite = await transformSaber(lp);
        fs.writeFileSync("../pools/saber/"+lp.addresses.swapAccount+".json", JSON.stringify(toWrite))
    }

    else if (provider == "raydium"){
        const toWrite = transformRaydium(lp);
        fs.writeFileSync("../pools/raydium/"+lp.poolCoinTokenAccount.toBase58()+".json", JSON.stringify(toWrite))

    }
else     if (provider == "orca"){
const toWrite = await transformData(lp);
fs.writeFileSync("../pools/orca/"+lp.pool.poolParams.address.toBase58()+".json", JSON.stringify(toWrite))


    }
else if (provider == "openbook"){
const toWrite = {
    "accountFlags": lp.accountFlags,
    "ownAddress": lp.ownAddress.toBase58(),
    "vaultSignerNonce": lp.vaultSignerNonce.toString(),
    "baseMint": lp.baseMint.toBase58(),
    "quoteMint": lp.quoteMint.toBase58(),
    "baseVault": lp.baseVault.toBase58(),
    "baseDepositsTotal": lp.baseDepositsTotal.toString(),
    "baseFeesAccrued": lp.baseFeesAccrued.toString(),
    "quoteVault": lp.quoteVault.toBase58(),
    "quoteDepositsTotal": lp.quoteDepositsTotal.toString(),
    "quoteFeesAccrued": lp.quoteFeesAccrued.toString(),
    "quoteDustThreshold": lp.quoteDustThreshold.toString(),
    "requestQueue": lp.requestQueue.toBase58(),
    "eventQueue": lp.eventQueue.toBase58(),
    "bids": lp.bids.toBase58(),
    "asks": lp.asks.toBase58(),
    "baseLotSize": lp.baseLotSize.toString(),
    "quoteLotSize": lp.quoteLotSize.toString(),
    "feeRateBps": lp.feeRateBps.toString(),
    "referrerRebatesAccrued": lp.referrerRebatesAccrued.toString(),
    "provider": lp.provider,
    "other": lp.baseMint.toBase58(),
    "baseScale": lp.baseLotSize.toNumber(),
    "quoteScale": lp.quoteLotSize.toNumber(),
}
fs.writeFileSync("../pools/openbook/"+lp.ownAddress.toBase58()+".json", JSON.stringify(toWrite))
  
}
}
    
}
async function main2(){
  const raydium = JSON.parse(fs.readFileSync("./mainnet.json"))
  const official = raydium.official 
  let unOfficial = raydium.unOfficial
  for (var pool of [...official, ...unOfficial]){
    fs.writeFileSync("../pools/raydium/"+pool.id+".json", JSON.stringify(pool))
  }
}
main2()