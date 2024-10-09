use super::erc20::*;
use traits::{Mintable, IERC20};

use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn e2e_transfer<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    let total_supply = 1_000_000_000;
    let mut constructor = ERC20Ref::new("test".to_owned(), "Test".to_owned());
    let erc20 = client
        .instantiate("erc20", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = erc20.call_builder::<ERC20>();
    let alice_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);
    let mint = call_builder.mint(alice_account, total_supply);
    let _transfer_res = client
        .call(&ink_e2e::alice(), &mint)
        .submit()
        .await
        .expect("mint failed");
    // when
    let total_supply_msg = call_builder.total_supply();
    let total_supply_res = client
        .call(&ink_e2e::bob(), &total_supply_msg)
        .dry_run()
        .await?;

    let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
    let transfer_to_bob = 500_000_000u128;
    let transfer = call_builder.transfer(bob_account, transfer_to_bob);
    let _transfer_res = client
        .call(&ink_e2e::alice(), &transfer)
        .submit()
        .await
        .expect("transfer failed");

    let balance_of = call_builder.balance_of(bob_account);
    let balance_of_res = client
        .call(&ink_e2e::alice(), &balance_of)
        .dry_run()
        .await?;

    // then
    assert_eq!(
        total_supply,
        total_supply_res.return_value(),
        "total_supply"
    );
    assert_eq!(transfer_to_bob, balance_of_res.return_value(), "balance_of");

    Ok(())
}

#[ink_e2e::test]
async fn e2e_allowances<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    // let total_supply = 1_000_000_000;
    // let mut constructor = ERC20Ref::new("test".to_owned(),"Test".to_owned());
    // let erc20 = client
    //     .instantiate("erc20", &ink_e2e::bob(), &mut constructor)
    //     .submit()
    //     .await
    //     .expect("instantiate failed");
    // let mut call_builder = erc20.call_builder::<ERC20>();

    // let alice_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);
    // let mint = call_builder.mint(alice_account, total_supply);
    // let _transfer_res = client
    //     .call(&ink_e2e::alice(), &mint)
    //     .submit()
    //     .await
    //     .expect("mint failed");
    // // when

    // let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
    // let charlie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie);

    // let amount = 500_000_000u128;
    // // tx
    // let transfer_from =
    //     call_builder.transfer_from(bob_account, charlie_account, amount);
    // let transfer_from_result = client
    //     .call(&ink_e2e::charlie(), &transfer_from)
    //     .submit()
    //     .await;

    // assert!(
    //     transfer_from_result.is_err(),
    //     "unapproved transfer_from should fail"
    // );

    // // Bob approves Charlie to transfer up to amount on his behalf
    // let approved_value = 1_000u128;
    // let approve_call = call_builder.approve(charlie_account, approved_value);
    // client
    //     .call(&ink_e2e::bob(), &approve_call)
    //     .submit()
    //     .await
    //     .expect("approve failed");

    // // `transfer_from` the approved amount
    // let transfer_from =
    //     call_builder.transfer_from(bob_account, charlie_account, approved_value);
    // let transfer_from_result = client
    //     .call(&ink_e2e::charlie(), &transfer_from)
    //     .submit()
    //     .await;
    // assert!(
    //     transfer_from_result.is_ok(),
    //     "approved transfer_from should succeed"
    // );

    // let balance_of = call_builder.balance_of(bob_account);
    // let balance_of_res = client
    //     .call(&ink_e2e::alice(), &balance_of)
    //     .dry_run()
    //     .await?;

    // // `transfer_from` again, this time exceeding the approved amount
    // let transfer_from =
    //     call_builder.transfer_from(bob_account, charlie_account, 1);
    // let transfer_from_result = client
    //     .call(&ink_e2e::charlie(), &transfer_from)
    //     .submit()
    //     .await;
    // assert!(
    //     transfer_from_result.is_err(),
    //     "transfer_from exceeding the approved amount should fail"
    // );

    // assert_eq!(
    //     total_supply - approved_value,
    //     balance_of_res.return_value(),
    //     "balance_of"
    // );

    Ok(())
}
