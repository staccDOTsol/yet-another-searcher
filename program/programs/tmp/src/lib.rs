use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_spl::token::TokenAccount;

declare_id!("CRQXfRGq3wTkjt7JkqhojPLiKLYLjHPGLebnfiiQB46T");

use error::ErrorCode;
use state::SwapState;

pub mod error;
pub mod ix_data;
pub mod state;
pub mod swaps;

pub use swaps::*;

#[program]
pub mod tmp {
    use super::*;

    pub fn init_program(ctx: Context<InitSwapState>) -> Result<()> {
        let swap_state = &mut ctx.accounts.swap_state;
        swap_state.swap_input = 0;
        swap_state.is_valid = false;
        Ok(())
    }

    pub fn start_swap(ctx: Context<TokenAndSwapState>, swap_input: u64) -> Result<()> {
        let swap_state = &mut ctx.accounts.swap_state;
        swap_state.swap_input = swap_input; // !
        swap_state.is_valid = true;
        Ok(())
    }

    pub fn profit_or_revert(ctx: Context<TokenAndSwapState>) -> Result<()> {
        let swap_state = &mut ctx.accounts.swap_state;
        swap_state.is_valid = false; // record end of swap



        Ok(())
    }

    /// Convenience API to initialize an open orders account on the Serum DEX.
    pub fn init_open_order(ctx: Context<InitOpenOrder>) -> Result<()> {
        _init_open_order(ctx)
    }

    pub fn orca_swap<'info>(ctx: Context<'_, '_, '_, 'info, OrcaSwap<'info>>) -> Result<()> {
        basic_pool_swap!(_orca_swap, OrcaSwap<'info>)(ctx)
    }

    pub fn mercurial_swap<'info>(
        ctx: Context<'_, '_, '_, 'info, MercurialSwap<'info>>,
    ) -> Result<()> {
        basic_pool_swap!(_mercurial_swap, MercurialSwap<'info>)(ctx)
    }

    pub fn saber_swap<'info>(ctx: Context<'_, '_, '_, 'info, SaberSwap<'info>>) -> Result<()> {
        basic_pool_swap!(_saber_swap, SaberSwap<'info>)(ctx)
    }

    pub fn aldrin_swap_v2<'info>(
        ctx: Context<'_, '_, '_, 'info, AldrinSwapV2<'info>>,
        is_inverted: bool,
    ) -> Result<()> {
        let amount_in = prepare_swap(&ctx.accounts.swap_state)?;

        _aldrin_swap_v2(&ctx, amount_in, is_inverted)?;

        // end swap
        let user_dst = match is_inverted {
            true => &mut ctx.accounts.user_quote_ata,
            false => &mut ctx.accounts.user_base_ata,
        };
        let swap_state = &mut ctx.accounts.swap_state;
        end_swap(swap_state, user_dst)?;

        Ok(())
    }

    pub fn aldrin_swap_v1<'info>(
        ctx: Context<'_, '_, '_, 'info, AldrinSwapV1<'info>>,
        is_inverted: bool,
    ) -> Result<()> {
        let amount_in = prepare_swap(&ctx.accounts.swap_state)?;

        _aldrin_swap_v1(&ctx, amount_in, is_inverted)?;

        // end swap
        let user_dst = match is_inverted {
            true => &mut ctx.accounts.user_quote_ata,
            false => &mut ctx.accounts.user_base_ata,
        };
        let swap_state = &mut ctx.accounts.swap_state;
        end_swap(swap_state, user_dst)?;

        Ok(())
    }

    pub fn openbook_swap<'info>(
        ctx: Context<'_, '_, '_, 'info, SerumSwap<'info>>,
        side: Side,
    ) -> Result<()> {
        let amount_in = prepare_swap(&ctx.accounts.swap_state)?;
        let is_bid = match side {
            Side::Bid => true,
            Side::Ask => false,
        };

        _serum_swap(&ctx, amount_in, side)?;

        // end swap
        let user_dst = match is_bid {
            true => &mut ctx.accounts.market.coin_wallet,
            false => &mut ctx.accounts.pc_wallet,
        };
        let swap_state = &mut ctx.accounts.swap_state;
        end_swap(swap_state, user_dst)?;

        Ok(())
    }
}

#[macro_export]
macro_rules! basic_pool_swap {
    ($swap_fcn:expr, $typ:ident < $tipe:tt > ) => {{
        |ctx: Context<'_, '_, '_, 'info, $typ<$tipe>>| -> Result<()> {
            // save the amount of input swap
            let amount_in = prepare_swap(&ctx.accounts.swap_state).unwrap();

            // do swap
            $swap_fcn(&ctx, amount_in).unwrap();

            // update the swap output amount (to be used as input to next swap)
            let swap_state = &mut ctx.accounts.swap_state;
            let user_dst = &mut ctx.accounts.user_dst;
            end_swap(swap_state, user_dst).unwrap();

            Ok(())
        }
    }};
}

pub fn end_swap(
    swap_state: &mut Account<SwapState>,
    user_dst: &mut Account<TokenAccount>,
) -> Result<()> {
    // derive the output of the swap
    let dst_start_balance = user_dst.amount; // pre-swap balance
    user_dst.reload()?; // update underlying account
    let dst_end_balance = user_dst.amount; // post-swap balance
    let swap_amount_out = dst_end_balance - dst_start_balance;
    msg!("swap amount out: {:?}", swap_amount_out);

    // will be input amount into the next swap ix
    swap_state.swap_input = swap_amount_out;

    Ok(())
}

pub fn prepare_swap(swap_state: &Account<SwapState>) -> Result<u64> {
    require!(swap_state.is_valid, ErrorCode::InvalidState);
    // get the swap in amount from the state
    let amount_in = swap_state.swap_input;
    msg!("swap amount in: {:?}", amount_in);

    Ok(amount_in)
}

#[derive(Accounts)]
pub struct InitSwapState<'info> {
    #[account(
        init, 
        space=17+8,
        payer=payer,
        seeds=[b"swap_state"], 
        bump, 
    )]
    pub swap_state: Account<'info, SwapState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TokenAndSwapState<'info> {
    #[account(mut, seeds=[b"swap_state"], bump)]
    pub swap_state: Account<'info, SwapState>,
}
