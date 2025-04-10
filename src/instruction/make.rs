use bytemuck::{from_bytes, Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_token::state::TokenAccount;

use crate::state::{DataLen, Escrow};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct MakeEscrowIx {
    pub bump: u8,
    pub amount_a: [u8; 8],
    pub amount_b: [u8; 8],
}

pub fn process_make(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, maker_ata, vault, escrow, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if data.len() < core::mem::size_of::<MakeEscrowIx>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let ix_data: &MakeEscrowIx = bytemuck::from_bytes(data);
    /*

    //without bytemuck and raw pointers:

    let bump = data[0];
    let amount_a = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let amount_b = data[9..17].try_into().unwrap();

     */

    let seed = [(b"escrow"), maker.key().as_slice(), &[ix_data.bump]];
    let seeds = &seed[..];

    let pda = pubkey::checked_create_program_address(seeds, &crate::ID).unwrap();
    assert_eq!(&pda, escrow.key());

    //is escrow the vault onwer?
    assert!(unsafe {
        TokenAccount::from_account_info_unchecked(vault)
            .unwrap()
            .owner()
            == escrow.key()
    });

    //has the escrow bee initialized?
    if unsafe { escrow.owner() } != &crate::ID {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Create Escrow Account
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: escrow,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }
    .invoke()?;

    // Populate Escrow Account
    Escrow::initialize(
        escrow,
        *maker.key(),
        *mint_a.key(),
        *mint_b.key(),
        ix_data.amount_b,
        ix_data.bump,
    );

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: vault,
        authority: maker,
        amount: u64::from_le_bytes(ix_data.amount_a),
    }
    .invoke()?;

    Ok(())
}
