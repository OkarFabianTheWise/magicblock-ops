# Account Delegation System

This module provides functionality to delegate and undelegate accounts in the Magic Pinocchio program.

## Delegation Process

### Overview
The delegation system allows accounts to temporarily transfer ownership to a delegation program while preserving their original state.

### Delegate Operation (`process_delegate`)
Transfers account ownership to the delegation program while preserving the account's data in a buffer.

#### Required Accounts
```rust
1. maker         - The account initiating the delegation (Signer)
2. pda_acc      - The account to be delegated
3. magic_acc    - The Magic Pinocchio program account
4. buffer_acc   - Temporary storage for account data
5. delegation_record    - Record of the delegation
6. delegation_metadata  - Metadata for the delegation
7. system_program      - System Program
```

#### Process
1. Creates a buffer account to store original account data
2. Copies all data from the original account to the buffer
3. Closes the original account (zeroes lamports and data)
4. Recreates the account under delegation program ownership
5. Initiates the delegation with configured parameters:
   - Commit frequency: 30 seconds (30,000ms)
   - Original account seeds
   - Optional validator

### Undelegate Operation (`process_undelegate`)
Restores the original account ownership and data from the buffer.

#### Required Accounts
```rust
1. maker         - The account initiating the undelegation (Signer)
2. pda_acc      - The delegated account
3. buffer_acc   - Buffer containing original account data
4. delegation_record    - Record of the delegation
5. delegation_metadata  - Metadata for the delegation
6. system_program      - System Program
```

#### Process
1. Closes the delegated account
2. Recreates the account with original program ownership
3. Restores original data from buffer
4. Closes the buffer account
5. Notifies delegation program of undelegation

## Usage Example

```rust
// Delegate an account
let delegate_instruction = Instruction {
    program_id: program_id,
    accounts: vec![
        maker.to_account_meta(),
        account_to_delegate.to_account_meta(),
        magic_program.to_account_meta(),
        buffer.to_account_meta(),
        delegation_record.to_account_meta(),
        delegation_metadata.to_account_meta(),
        system_program.to_account_meta(),
    ],
    data: /* delegation parameters */,
};

// Undelegate an account
let undelegate_instruction = Instruction {
    program_id: program_id,
    accounts: vec![
        maker.to_account_meta(),
        delegated_account.to_account_meta(),
        buffer.to_account_meta(),
        delegation_record.to_account_meta(),
        delegation_metadata.to_account_meta(),
        system_program.to_account_meta(),
    ],
    data: /* undelegation parameters */,
};
```

## Security Considerations

- All account ownership changes are performed through CPIs
- Original account data is preserved in a secure buffer
- Only the original owner can initiate delegation/undelegation
- Uses PDAs with proper seeds for security
- Proper cleanup of buffer accounts after undelegation

## Constants

```rust
pub const DELEGATION_ACCOUNT: Pubkey = 
    pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");
```