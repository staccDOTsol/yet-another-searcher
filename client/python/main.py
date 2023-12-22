from ray import serve

from typing import List
from base64 import b64decode
import struct
from typing import Tuple
import os
import json
from abc import ABC, abstractmethod
from enum import Enum
from typing import List, Tuple
import ray
from pyserum.connection import conn
from pyserum.market.state import MarketState
from pyserum.market import Market
cc = conn("https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718")
class PoolType(Enum):
    OrcaPoolType = 1
    MercurialPoolType = 2
    SaberPoolType = 3
    AldrinPoolType = 4
    SerumPoolType = 5
    RaydiumPoolType = 6

class PoolType2:
    RaydiumPoolType = "Raydium"
    OrcaPoolType = "Orca"
    SaberPoolType = "Saber"
    SerumPoolType = "Serum"

class PoolDir:
    def __init__(self, pool_type, dir_path):
        self.pool_type = pool_type
        self.dir_path = dir_path

class PoolOperations(ABC):

    @abstractmethod
    def get_own_address(self):
        pass

    @abstractmethod
    def set_update_accounts2(self, pubkey: str, data):
        pass


    @abstractmethod
    def get_quote_with_amounts_scaled(self, amount_in: int, mint_in: str, mint_out: str) -> int:
        pass
    
    @abstractmethod
    def get_mints(self):
        pass
    
    
    

def read_json_dir(dir: str) -> List[str]:
    return [os.path.join(dir, f) for f in os.listdir(dir) if f.endswith('.json')]


N_COINS = 2
N_COINS_SQUARED = 4
ITERATIONS = 32
from decimal import Decimal, getcontext

getcontext().prec = 28

N_COINS = 2
ITERATIONS = 32
def checked_u8_power(a, b):
    return a ** b

def checked_u8_mul(a, b):
    return a * b

def compute_new_destination_amount(leverage, new_source_amount, d_val):
    # Upscale to Decimal
    leverage = Decimal(leverage)
    new_source_amount = Decimal(new_source_amount)
    d_val = Decimal(d_val)

    # sum' = prod' = x
    # c =  D ** (n + 1) / (n ** (2 * n) * prod' * A)
    c = checked_u8_power(d_val, N_COINS + 1) / (checked_u8_mul(new_source_amount, N_COINS_SQUARED) * leverage)

    # b = sum' - (A*n**n - 1) * D / (A * n**n)
    b = new_source_amount + d_val / leverage

    # Solve for y by approximating: y**2 + b*y = c
    y = d_val
    for _ in range(ITERATIONS):
        y_new = (checked_u8_power(y, 2) + c) / (checked_u8_mul(y, 2) + b - d_val)
        if y_new == y:
            break
        else:
            y = y_new
    return int(y)
def calculate_step(d, leverage, sum_x, d_product):
    return (leverage * sum_x + d_product * N_COINS) * d // ((leverage - 1) * d + (N_COINS + 1) * d_product)

def compute_a(amp):
    return amp * N_COINS

def compute_d(leverage, amount_a, amount_b):
    amount_a_times_coins = checked_u8_mul(Decimal(amount_a), N_COINS) + 1
    amount_b_times_coins = checked_u8_mul(Decimal(amount_b), N_COINS) + 1
    sum_x = amount_a + amount_b  # sum(x_i), a.k.a S
    if sum_x == 0:
        return 0
    else:
        d_previous = 0
        d = Decimal(sum_x)

        # Newton's method to approximate D
        for _ in range(ITERATIONS):
            d_product = d
            d_product = d_product * d / amount_a_times_coins
            d_product = d_product * d / amount_b_times_coins
            d_previous = d
            d = calculate_step(d, leverage, sum_x, d_product)
            # Equality with the precision of 1
            if d == d_previous:
                break
        return int(d)
class Stable:
    def __init__(self, amp, fee_numerator, fee_denominator):
        self.amp = amp
        self.fee_numerator = fee_numerator
        self.fee_denominator = fee_denominator

    def get_quote(self, pool_amounts, precision_multipliers, scaled_amount_in):
        xp = [
            pool_amounts[0] * precision_multipliers[0],
            pool_amounts[1] * precision_multipliers[1],
        ]
        dx = scaled_amount_in * precision_multipliers[0]

        x = xp[0] + dx
        leverage = compute_a(self.amp)
        d = compute_d(leverage, xp[0], xp[1])
        
        y = compute_new_destination_amount(leverage, x, d)
        if y is None:
            return 0
        dy = xp[1] - y
        out_amount = dy // precision_multipliers[1]

        # reduce fees at the end
        fees = (out_amount * self.fee_numerator) // self.fee_denominator

        return out_amount - fees
class SaberPool(PoolOperations):
    def get_mints(self):
        return self.token_ids
    def get_own_address(self):
        return self.pool_account
    def __init__(self, pool_account, pool_token_mint, decimals, fee_accounts, token_ids, tokens, fee_numerator, fee_denominator, target_amp, *args, **kwargs):
        self.pool_account = pool_account
        self.pool_token_mint = pool_token_mint
        self.pool_token_decimals = decimals
        self.fee_accounts = fee_accounts
        self.token_ids = token_ids
        self.tokens = tokens
        self.fee_structure = {
            "owner_fee": {
                "numerator": fee_numerator,
                "denominator": fee_denominator
            },
            "trader_fee": {
                "numerator": fee_numerator,
                "denominator": fee_denominator
            }
        }
        self.target_amp = target_amp
        self.pool_amounts = {}
    def get_quote_with_amounts_scaled(self, scaled_amount_in, mint_in, mint_out):
        try:
            calculator = Stable(self.target_amp, self.fee_numerator, self.fee_denominator)
            pool_src_amount = self.pool_amounts.get(str(mint_in))
            pool_dst_amount = self.pool_amounts.get(str(mint_out))
            pool_amounts = [pool_src_amount, pool_dst_amount]
            precision_multipliers = [1, 1]

            return calculator.get_quote(pool_amounts, precision_multipliers, scaled_amount_in)
        except Exception as e:
            print(e)
            return 0
    def set_update_accounts2(self, pubkey: str, data):
        account = data_store[pubkey]
        id0 = self.token_ids[0]
        id1 = self.token_ids[1]
        if str(account.mint) == str(id0):
            if str(id0) in self.pool_amounts:
                del self.pool_amounts[str(id0)]
            self.pool_amounts[str(id0)] = account.amount
        elif str(account.mint) == str(id1):
            if str(id1) in self.pool_amounts:
                del self.pool_amounts[str(id1)]
            self.pool_amounts[str(id1)] = account.amount
class CurveType:
    ConstantProduct = 0
    Stable = 2

class Fees:
    def __init__(self, trade_fee_numerator, trade_fee_denominator, owner_trade_fee_numerator, owner_trade_fee_denominator):
        self.trade_fee_numerator = trade_fee_numerator
        self.trade_fee_denominator = trade_fee_denominator
        self.owner_trade_fee_numerator = owner_trade_fee_numerator
        self.owner_trade_fee_denominator = owner_trade_fee_denominator
        self.owner_withdraw_fee_numerator = 0
        self.owner_withdraw_fee_denominator = 0
        self.host_fee_numerator = 0
        self.host_fee_denominator = 0
from enum import Enum

class CurveType(Enum):
    ConstantProduct = 0
    Stable = 2

class TradeDirection(Enum):
    AtoB = 0
    BtoA = 1
from abc import ABC, abstractmethod

class CurveCalculator(ABC):
    @abstractmethod
    def calculate(self):
        pass

class StableCurve(CurveCalculator):
    def __init__(self, amp):
        self.amp = amp

    def calculate(self):
        # Implement calculation logic here
        pass

class SwapCurve:
    def __init__(self, curve_type, calculator):
        self.curve_type = curve_type
        self.calculator = calculator
def get_pool_quote_with_amounts(amount_in, curve_type, amp, fees, input_token_pool_amount, output_token_pool_amount, slippage_percent=None):
    quote = None
    trade_direction = TradeDirection.AtoB

    if curve_type == CurveType.ConstantProduct:# ??? pass pass pass ? 
        # constant product (1 for orca)
        swap_curve = SwapCurve(
            curve_type=CurveType.ConstantProduct,
            calculator=StableCurve(amp) # not sure
        )
        swap_quote = swap_curve.swap(
            amount_in,
            input_token_pool_amount,
            output_token_pool_amount,
            trade_direction,
            fees
        )
        quote = swap_quote.destination_amount_swapped if swap_quote else 0
    elif curve_type == CurveType.Stable:
        # stableswap (2 for orca)
        swap_curve = SwapCurve(
            curve_type=CurveType.Stable,
            calculator=StableCurve(amp)
        )
        swap_quote = swap_curve.swap(
            amount_in,
            input_token_pool_amount,
            output_token_pool_amount,
            trade_direction,
            fees
        )
        quote = swap_quote.destination_amount_swapped
    else:
        raise Exception(f"invalid curve type for swap: {curve_type}")

    # add slippage amount if its given
    
    return quote
class OrcaPool(PoolOperations):
    def get_mints(self):
        return self.token_ids
    
    def get_own_address(self):
        return self.address
    def __init__(self, address, nonce, authority, poolTokenMint, poolTokenDecimals, feeAccount, tokenIds, tokens, feeStructure, curveType, *args, **kwargs):
        self.address = address
        self.nonce = nonce
        self.authority = authority
        self.pool_token_mint = poolTokenMint
        self.pool_token_decimals = poolTokenDecimals
        self.fee_account = feeAccount
        self.token_ids = tokenIds
        self.tokens = tokens
        self.fee_structure = feeStructure
        self.curve_type = curveType
        self.amp = 1
        self.pool_amounts = {}
    

    def get_quote_with_amounts_scaled(self, scaled_amount_in, mint_in, mint_out):
        try:
            pool_src_amount = self.pool_amounts.get(str(mint_in))
            pool_dst_amount = self.pool_amounts.get(str(mint_out))

            # compute fees
            trader_fee = self.fee_structure.trader_fee
            owner_fee = self.fee_structure.owner_fee
            fees = Fees(
                trader_fee.numerator,
                trader_fee.denominator,
                owner_fee.numerator,
                owner_fee.denominator
            )

            if self.curve_type == 0:
                ctype = CurveType.ConstantProduct
            elif self.curve_type == 2:
                ctype = CurveType.Stable
            else:
                raise Exception(f"invalid self curve type: {self.curve_type}")

            # get quote -- works for either constant product or stable swap
            amt = get_pool_quote_with_amounts(
                scaled_amount_in,
                ctype,
                self.amp,
                fees,
                pool_src_amount,
                pool_dst_amount,
                None
            )
            print('orca')
            if amt > 0:
                return amt - 1
            else:
                return amt
        except Exception as e:
            print(e)
            return 0
    def set_update_accounts2(self, pubkey: str, data):
        data = data_store[pubkey]
        account = unpack_token_account(data)
        id0 = self.token_ids[0]
        id1 = self.token_ids[1]
        if str(account.mint) == id0:
            self.pool_amounts.pop(id0, None)
            self.pool_amounts[id0] = account.amount
        elif str(account.mint) == id1:
            self.pool_amounts.pop(id1, None)
            self.pool_amounts[id1] = account.amount
class Iteration:
    def __init__(self, amount_in, amount_out):
        self.amount_in = amount_in
        self.amount_out = amount_out

class FeeTier:
    def __init__(self, taker_fee):
        self.taker_fee = taker_fee

    def remove_taker_fee(self, amount):
        return amount - self.taker_fee

class Order:
    def __init__(self, price, quantity, order_id):
        self.price = price
        self.quantity = quantity
        self.order_id = order_id

    def set_quantity(self, quantity):
        self.quantity = quantity

class OrderBook:
    def __init__(self, orders):
        self.orders = orders

    def find_min(self):
        return min(self.orders, key=lambda order: order.price)

    def find_max(self):
        return max(self.orders, key=lambda order: order.price)

    def remove_by_key(self, order_id):
        self.orders = [order for order in self.orders if order.order_id != order_id]
class OrderBookState:
    def __init__(self, market_state, asks, bids):
        self.market_state = market_state
        self.asks = OrderBook(asks)
        self.bids = OrderBook(bids)

    
def bid_iteration(self, iteration, fee_tier):
    quote_lot_size = self.market_state.pc_lot_size
    base_lot_size = self.market_state.coin_lot_size
    if quote_lot_size == 0:
        quote_lot_size = 1
    start_amount_in = iteration.amount_in
    max_pc_qty = fee_tier.remove_taker_fee(iteration.amount_in) / quote_lot_size
    pc_qty_remaining = max_pc_qty

    while True:
        best_ask = self.asks.find_min()
        if best_ask is None:
            break
        trade_price = best_ask.price
        offer_size = best_ask.quantity
        trade_qty = min(offer_size, pc_qty_remaining / trade_price)

        if trade_qty == 0 or offer_size == 0:
            break

        pc_qty_remaining -= trade_qty * trade_price
        iteration.amount_out += trade_qty * base_lot_size

        best_ask.set_quantity(best_ask.quantity - trade_qty)

        if best_ask.quantity == 0:
            self.asks.remove_by_key(best_ask.order_id)

    native_accum_fill_price = (max_pc_qty - pc_qty_remaining) * quote_lot_size
    native_taker_fee = fee_tier.taker_fee(native_accum_fill_price)
    native_pc_qty_remaining = start_amount_in - native_accum_fill_price - native_taker_fee
    iteration.amount_in = native_pc_qty_remaining

    return pc_qty_remaining == 0

def ask_iteration(self, iteration, fee_tier):
    pc_lot_size = self.market_state.pc_lot_size
    coin_lot_size = self.market_state.coin_lot_size
    if coin_lot_size == 0:
        coin_lot_size = 1
    max_qty = iteration.amount_in
    unfilled_qty = max_qty / coin_lot_size
    accum_fill_price = 0

    while True:
        best_bid = self.bids.find_max()
        if best_bid is None:
            break
        trade_price = best_bid.price
        bid_size = best_bid.quantity
        trade_qty = min(bid_size, unfilled_qty)

        if trade_qty == 0 or bid_size == 0:
            break

        best_bid.set_quantity(best_bid.quantity - trade_qty)
        unfilled_qty -= trade_qty
        accum_fill_price += trade_qty * trade_price

        if best_bid.quantity == 0:
            self.bids.remove_by_key(best_bid.order_id)

    native_taker_pc_qty = accum_fill_price * pc_lot_size
    native_taker_fee = fee_tier.taker_fee(native_taker_pc_qty)
    net_taker_pc_qty = native_taker_pc_qty - native_taker_fee

    iteration.amount_out += net_taker_pc_qty
    iteration.amount_in = unfilled_qty * coin_lot_size

    return unfilled_qty == 0
from pyserum.async_connection import async_conn
from pyserum.market import AsyncMarket


import pyserum

class RaydiumPool(PoolOperations):
    # camelCase
    def get_mints(self):
        return [self.base_mint, self.quote_mint]
    
    def get_own_address(self):
        return self.id 
    def __init__(self, id, baseMint, quoteMint, lpMint, baseDecimals, quoteDecimals, lpDecimals, version, programId, authority, openOrders, targetOrders, baseVault, quoteVault, withdrawQueue, lpVault, marketVersion, marketProgramId, marketId, marketAuthority, marketBaseVault, marketQuoteVault, marketBids, marketAsks, marketEventQueue, *args, **kwargs):
        self.id = id
        self.base_mint = baseMint
        self.quote_mint = quoteMint
        self.lp_mint = lpMint
        self.base_decimals = baseDecimals
        self.quote_decimals = quoteDecimals
        self.lp_decimals = lpDecimals
        self.version = version
        self.program_id = programId
        self.authority = authority
        self.open_orders = openOrders
        self.target_orders = targetOrders
        self.base_vault = baseVault
        self.quote_vault = quoteVault
        self.withdraw_queue = withdrawQueue
        self.lp_vault = lpVault
        self.market_version = marketVersion
        self.market_program_id = marketProgramId
        self.market_id = marketId
        self.market_authority = marketAuthority
        self.market_base_vault = marketBaseVault
        self.market_quote_vault = marketQuoteVault
        self.market_bids = marketBids
        self.market_asks = marketAsks
        self.market_event_queue = marketEventQueue
        self.accounts = [id, marketBids, marketAsks]
    def get_quote_with_amounts_scaled(self, scaled_amount_in, mint_in, which):
        try:
            iteration = {'amount_in': scaled_amount_in, 'amount_out': 0}

            market_acc = self.id 
            
            market = Market()            
            market = market.load(cc, PublicKey(market_acc), PublicKey("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"))
            asks = market.load_asks()

            print(asks)
            bids = market.load_bids()
            # Show all current ask order

            ob = OrderBookState(bids, asks, market)

            if mint_in == self.quote_mint:
                # bid: quote -> base
                count = 0
                while True:
                    count += 1
                    done = bid_iteration(iteration, ob)
                    if done or iteration['amount_out'] == 0 or count == 5:
                        break
                return iteration['amount_out']
            elif mint_in == self.base_mint:
                # ask: base -> quote
                count = 0
                while True:
                    count += 1
                    done = ask_iteration(iteration, ob)
                    if done or iteration['amount_in'] == 0 or count == 5:
                        break
                return iteration['amount_out']
            else:
                print(0)
                return 0
        except Exception as e:
            print(e)
            return 0
    def set_update_accounts2(self, pubkey, data):
        data = data_store[pubkey]
        taccs = self.accounts.copy()
        if not taccs or len(taccs) < 3:
            return

        flags = account_flags(data)  # You need to implement this method
        if not flags:
            return

        if 'Bids' in flags:
            if len(taccs) < 2:
                taccs.append(Account(data=data))
            bids = taccs[1]
            bids.data = data
            if len(taccs) < 3:
                self.accounts = [taccs[0], bids]
            else:
                self.accounts = [taccs[0], bids, taccs[2]]

        if 'Asks' in flags:
            if len(taccs) < 3:
                taccs.append(Account(data=data))
                self.accounts = [taccs[0], taccs[1], taccs[2]]
            else:
                asks = taccs[2]
                asks.data = data
                self.accounts = [taccs[0], taccs[1], asks]
class Account:
    def __init__(self, lamports=0, data=None, owner=None, executable=False, rent_epoch=0):
        self.lamports = lamports
        self.data = data
        self.owner = owner
        self.executable = executable
        self.rent_epoch = rent_epoch
import solana.publickey as PublicKey
class SerumPool(PoolOperations):
    def get_mints(self):
        return [self.base_mint, self.quote_mint]
    def get_own_address(self):
        return self.own_address 
    def __init__(self, ownAddress, baseMint, quoteMint, baseScale, quoteScale, baseVault, quoteVault, requestQueue, eventQueue, bids, asks, vaultSignerNonce, feeRateBps, *args, **kwargs):
        self.own_address = ownAddress
        self.base_mint = baseMint
        self.quote_mint = quoteMint
        self.base_scale = baseScale
        self.quote_scale = quoteScale
        self.base_vault = baseVault
        self.quote_vault = quoteVault
        self.request_queue = requestQueue
        self.event_queue = eventQueue
        self.bids = bids
        self.asks = asks
        self.vault_signer_nonce = vaultSignerNonce
        self.fee_rate_bps = feeRateBps
        self.accounts = []
        self.open_orders = None

    def get_quote_with_amounts_scaled(self, scaled_amount_in, mint_in, which):
        try:
            iteration = {'amount_in': scaled_amount_in, 'amount_out': 0}

            market_acc = self.own_address 

            market = Market()            
            market = market.load(cc, PublicKey(market_acc), PublicKey("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"))
            asks = market.load_asks()
            print(asks)
            bids = market.load_bids()
            # Show all current ask order

            ob = OrderBookState(bids, asks, market)

            if mint_in == self.quote_mint:
                # bid: quote -> base
                count = 0
                while True:
                    count += 1
                    done = bid_iteration(iteration, ob)
                    if done or iteration['amount_out'] == 0 or count == 5:
                        break
                return iteration['amount_out']
            elif mint_in == self.base_mint:
                # ask: base -> quote
                count = 0
                while True:
                    count += 1
                    done = ask_iteration(iteration, ob)
                    if done or iteration['amount_in'] == 0 or count == 5:
                        break
                return iteration['amount_out']
            else:
                print(0)
                return 0
        except Exception as e:
            print(e)
            return 0
    def set_update_accounts2(self, _pubkey, data):
        taccs = self.accounts.copy()
        if not taccs or len(taccs) < 3:
            return

        flags = account_flags(data)  # You need to implement this method
        if not flags:
            return

        if 'Bids' in flags:
            if len(taccs) < 2:
                taccs.append(Account(data=data))
            bids = taccs[1]
            bids.data = data
            if len(taccs) < 3:
                self.accounts = [taccs[0], bids]
            else:
                self.accounts = [taccs[0], bids, taccs[2]]

        if 'Asks' in flags:
            if len(taccs) < 3:
                taccs.append(Account(data=data))
                self.accounts = [taccs[0], taccs[1], taccs[2]]
            else:
                asks = taccs[2]
                asks.data = data
                self.accounts = [taccs[0], taccs[1], asks]
data_store = {}
results = {}

from enum import IntFlag
import struct

class PoolOperations2:
    def __init__(self):
        self.pool_dirs = []
        self.pools = []
    def add_pool_dir(self, pool_type, dir_path):
        pool_dir = PoolDir(pool_type, dir_path)
        self.pool_dirs.append(pool_dir)
    def get_pools(self):
        return self.pools
    def process_pool_dirs(self):
        for pool_dir in self.pool_dirs:
            
            print(f"pool dir: {pool_dir.dir_path}")
            pool_paths = self.read_json_dir(pool_dir.dir_path)

            for pool_path in pool_paths:
                with open(pool_path, 'r') as f:
                    json_str = f.read()
                pool = self.pool_factory(pool_dir.pool_type, json_str)
                self.pools.append(pool)
    def read_json_dir(self, dir_path):
        return [os.path.join(dir_path, f) for f in os.listdir(dir_path) if f.endswith('.json')]

    def pool_factory(self, pool_type, json_str):
        pool_data = json.loads(json_str)
        if pool_type == PoolType.RaydiumPoolType:
            return RaydiumPool(**pool_data)
        elif pool_type == PoolType.OrcaPoolType:
            return OrcaPool(**pool_data)
        elif pool_type == PoolType.SaberPoolType:
            return SaberPool(**pool_data)
        elif pool_type == PoolType.SerumPoolType:
            return SerumPool(**pool_data)
        else:
            raise Exception("Invalid pool type")

pool_operations = PoolOperations2()
pool_operations.add_pool_dir(PoolType.RaydiumPoolType, "../../pools/raydium")
pool_operations.add_pool_dir(PoolType.OrcaPoolType, "../../pools/orca")
pool_operations.add_pool_dir(PoolType.SaberPoolType, "../../pools/saber")
pool_operations.add_pool_dir(PoolType.SerumPoolType, "../../pools/openbook")
pool_operations.process_pool_dirs()
class AccountFlag(IntFlag):
    Initialized = 1 << 0
    Market = 1 << 1
    OpenOrders = 1 << 2
    RequestQueue = 1 << 3
    EventQueue = 1 << 4
    Bids = 1 << 5
    Asks = 1 << 6
    Disabled = 1 << 7
    Closed = 1 << 8
    Permissioned = 1 << 9
    CrankAuthorityRequired = 1 << 10

ACCOUNT_HEAD_PADDING = b''  # You need to define this

def account_flags(account_data):
    start = len(ACCOUNT_HEAD_PADDING)
    end = start + struct.calcsize('Q')  # 'Q' is for u64 in Rust

    if len(account_data) < end:
        raise ValueError('InvalidMarketFlags')

    flag_bytes = account_data[start:end]
    flags = struct.unpack('Q', flag_bytes)[0]

    try:
        return AccountFlag(flags)
    except ValueError:
        raise ValueError('InvalidMarketFlags')

class ShyftHtml2:
    def __init__(self, account):
        self.account = account
def unpack_token_account(data: bytes) -> Tuple[int, int]:
    if len(data) != 165:
        return (0, 0)

    # Unpack the mint and amount from the byte array
    mint = struct.unpack_from('<Q', data, 0)[0]
    amount = struct.unpack_from('<Q', data, 32)[0]

    return mint, amount
class Shyft:
    
    
    def __init__(self):
        pass
    
    

        
        
from ray import serve

import requests
from starlette.requests import Request
from typing import Dict
from fastapi import FastAPI, BackgroundTasks
from starlette.requests import Request
from starlette.responses import JSONResponse
from time import sleep

ray.init("212.6.53.200:6379")
app = FastAPI()

import asyncio 
@app.post("/shyft")
async def process(request: Request):
    try:
        json_data = await request.json()
        # Your processing logic here
        await (process_data_shyft(json_data))
        return {"result": "success"}
    except Exception as e:
        print(e)
        return {"result": "this is a teapot"}

@app.get("/shyft")
def get_data_shyft():
    global doing, pool_operations, results
    
    rs = ([ray.get(result) for result in results.values()])
    print(rs)
    largest_output = max(rs)
    print(largest_output)
    return {"largest arb for $100": largest_output, "results": rs}
doing = False
async def process_data_shyft(jsony):
    global pool_operations, data_store, results
    # write account to file
    for account in jsony['accounts']:
        pubkey = account['address']
        data = b64decode(account['data'])

        data_store[pubkey] = bytes(data)
        
        po = pool_operations.get_pools()
        print(pubkey)
        for pool in po:
            if pubkey == pool.get_own_address():
                
                pool.set_update_accounts2(pubkey, data)
                mint_pairs = pool.get_mints()
                result = pool.get_quote_with_amounts_scaled(100 * 10 ** 6, mint_pairs[0],mint_pairs[1])
                print(result)
                results[pubkey] = ray.put(result)
    # Your processing logic here
    return {"result": "success"}