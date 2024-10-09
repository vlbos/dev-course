
#![cfg_attr(not(feature = "std"), no_std, no_main)]
//  use core::error::Error;
use ink::prelude::{string::String};
type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;
type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
// type Error = <ink::env::DefaultEnvironment as ink::env::Environment>::Error;
//  pub type Result<T> = core::result::Result<T, Error>;
 /// The ERC-20 error types.
    // #[derive(Debug, PartialEq, Eq)]
    // #[ink::scale_derive(Encode, Decode, TypeInfo)]
    // pub enum Error {
    //     /// Returned if not enough balance to fulfill a request is available.
    //     InsufficientBalance,
    //     /// Returned if not enough allowance to fulfill a request is available.
    //     InsufficientAllowance,
    // }
//  pub type Result<T> = core::result::Result<T,  Error>;
    /// Trait implemented by all ERC-20 respecting smart contracts.
    #[ink::trait_definition]
    pub trait IERC20 {
        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance;

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(),String>;

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(),String>;

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(),String>;
    }


    /// Trait implemented by all ERC-20 Meta respecting smart contracts.
    #[ink::trait_definition]
    pub trait IERC20Meta {
        /// Returns the total token supply.
        #[ink(message)]
        fn name(&self) -> String;

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn symbol(&self) -> String;

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        fn decimals(&self) -> u8;
        
    }

    /// /// Trait implemented by Contract module which provides a basic access control mechanism, where
//  * there is an account (an owner) that can be granted exclusive access to
//  * specific functions.
//  *
//  * The initial owner is set to the address provided by the deployer. 
    #[ink::trait_definition]
    pub trait Ownable {
        /// Returns the address of the current owner.
        #[ink(message)]
        fn owner(&self) -> AccountId;

    }

    #[ink::trait_definition]
    pub trait Burnable {
        /// Destroys a `value` amount of tokens from the caller.
        #[ink(message)]
        fn burn(&mut self,value:Balance)->Result<(),String>;
    }

    #[ink::trait_definition]
    pub trait Mintable {
        /// Creates a `value` amount of tokens and assigns them to `account`
        #[ink(message)]
        fn mint(&mut self,to:AccountId,value:Balance)->Result<(),String>;
    }