use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

use super::DataLen;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Escrow {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount: [u8; 8],
    pub bump: u8,
}

impl DataLen for Escrow {
    const LEN: usize = core::mem::size_of::<Escrow>();
}

impl Escrow {
    pub fn initialize(
        escrow_acc: &AccountInfo,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        amount: [u8; 8],
        bump: u8,
    ) {
        let escrow =
            unsafe { &mut *(escrow_acc.borrow_mut_data_unchecked().as_ptr() as *mut Self) };

        escrow.maker = maker;
        escrow.mint_a = mint_a;
        escrow.mint_b = mint_b;
        escrow.amount = amount;
        escrow.bump = bump;
    }
}
