#[cfg(test)]
mod tests {

    #![no_std]
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;
    use mollusk_svm::{program, result::Check, Mollusk};
    use pinocchio_log::log;
    use solana_sdk::{
        account::{Account, AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_option::COption,
        program_pack::Pack,
        pubkey,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    };
    use spl_token::state::AccountState;

    const ID: Pubkey = pubkey!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");
    const DELEGATION_ACCOUNT: Pubkey = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

    #[test]
    fn test_make() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes()],
            &ID,
        );
        log!("bump test {}", escrow_bump);
        let escrow_account = Account::new(0, 0, &system_program);

        let mint_x = Pubkey::new_from_array([0x03; 32]);
        let mut mint_x_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        let mint_y = Pubkey::new_from_array([0x04; 32]);
        let mut mint_y_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        let maker_ata = Pubkey::new_from_array([0x05; 32]);
        let mut maker_ata_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: maker,
                amount: 100_000_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_account.data_as_mut_slice(),
        )
        .unwrap();

        let vault = Pubkey::new_from_array([0x06; 32]);
        let mut vault_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: escrow,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        let data = [
            vec![0],
            vec![escrow_bump],
            1_000_000u64.to_le_bytes().to_vec(),
            1_000_000u64.to_le_bytes().to_vec(),
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            ID,
            &data,
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new_readonly(mint_x, false),
                AccountMeta::new_readonly(mint_y, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(escrow, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(token_program, false),
            ],
        );

        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account),
                (mint_x, mint_x_account),
                (mint_y, mint_y_account),
                (maker_ata, maker_ata_account),
                (vault, vault_account),
                (escrow, escrow_account),
                (system_program, system_account),
                (token_program, token_account),
            ],
            &[Check::success()],
        );
    }

    #[test]
    fn test_take() {
        //TEST unfinished
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        let mint_x = Pubkey::new_from_array([0x03; 32]);
        let mut mint_x_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();
    }

    #[test]
    fn test_delegate() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        // Setup maker account
        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Setup PDA account
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes()],
            &ID,
        );
        let mut escrow_account = Account::new(mollusk.sysvars.rent.minimum_balance(100), 100, &ID);
        // Add some test data to escrow
        escrow_account.data_mut()[0] = 42;

        // Setup buffer account
        let (buffer, buffer_bump) =
            solana_sdk::pubkey::Pubkey::find_program_address(&[(b"buffer"), escrow.as_ref()], &ID);
        let buffer_account = Account::new(0, 0, &system_program);

        // Setup delegation accounts
        let delegation_record = Pubkey::new_from_array([0x07; 32]);
        let delegation_record_account = Account::new(0, 0, &system_program);

        let delegation_metadata = Pubkey::new_from_array([0x08; 32]);
        let delegation_metadata_account = Account::new(0, 0, &system_program);

        let magic_program = ID;

        let instruction = Instruction::new_with_bytes(
            ID,
            &[1], // Delegate instruction
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new(escrow, false),
                AccountMeta::new_readonly(magic_program, false),
                AccountMeta::new(buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new_readonly(delegation_metadata, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account.clone()),
                (escrow, escrow_account),
                (magic_program, Account::default()),
                (buffer, buffer_account),
                (delegation_record, delegation_record_account),
                (delegation_metadata, delegation_metadata_account),
                (system_program, system_account.clone()),
            ],
            &[Check::success()],
        );
    }

    #[test]
    fn test_undelegate() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        // Setup maker account
        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Setup delegated PDA account
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes()],
            &ID,
        );
        let escrow_account = Account::new(
            mollusk.sysvars.rent.minimum_balance(100),
            100,
            &DELEGATION_ACCOUNT, // Note: owned by delegation program
        );

        // Setup buffer account with stored data
        let (buffer, buffer_bump) =
            solana_sdk::pubkey::Pubkey::find_program_address(&[(b"buffer"), escrow.as_ref()], &ID);
        let mut buffer_account = Account::new(mollusk.sysvars.rent.minimum_balance(100), 100, &ID);
        // Add original data to buffer
        buffer_account.data_mut()[0] = 42;

        // Setup delegation accounts
        let delegation_record = Pubkey::new_from_array([0x07; 32]);
        let delegation_record_account = Account::new(0, 0, &system_program);

        let delegation_metadata = Pubkey::new_from_array([0x08; 32]);
        let delegation_metadata_account = Account::new(0, 0, &system_program);

        let instruction = Instruction::new_with_bytes(
            ID,
            &[2], // Undelegate instruction
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new(escrow, false),
                AccountMeta::new(buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new_readonly(delegation_metadata, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account.clone()),
                (escrow, escrow_account),
                (magic_program, Account::default()),
                (buffer, buffer_account),
                (delegation_record, delegation_record_account),
                (delegation_metadata, delegation_metadata_account),
                (system_program, system_account.clone()),
            ],
            &[
                Check::success(),
                Check::account_owner(escrow, &DELEGATION_ACCOUNT),
                Check::account_data_len(buffer, 100),
            ],
        );
    }
}
