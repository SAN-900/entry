use anchor_lang::prelude::Pubkey;
use entry::instructions::init_config::validate_upgrade_authority;

#[test]
fn accepts_the_upgrade_authority() {
    let authority = Pubkey::new_unique();

    assert!(validate_upgrade_authority(authority, Some(authority)).is_ok());
}

#[test]
fn rejects_a_different_signer() {
    let admin = Pubkey::new_unique();
    let upgrade_authority = Pubkey::new_unique();

    assert!(validate_upgrade_authority(admin, Some(upgrade_authority)).is_err());
}

#[test]
fn rejects_an_immutable_program() {
    let admin = Pubkey::new_unique();

    assert!(validate_upgrade_authority(admin, None).is_err());
}
