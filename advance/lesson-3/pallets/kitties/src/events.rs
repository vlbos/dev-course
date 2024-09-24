use frame_support::pallet_macros::pallet_section;

/// Define all events used in the pallet.
#[pallet_section]
mod events {
    #[pallet::event]
    // #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated {
            creator: T::AccountId,
            kitty_id: u32,
            data: [u8; 16],
        },
        KittyTransferred {
            from: T::AccountId,
            to: T::AccountId,
            kitty_id: u32,
        },
        KittyOnSale {
            owner: T::AccountId,
            kitty_id: u32,
            until_block: BlockNumberFor<T>,
        },
        KittyBid {
            bidder: T::AccountId,
            kitty_id: u32,
            price: BalanceOf<T>,
        },
        KittyTransferredAfterBidKnockedDown {
            from: T::AccountId,
            to: T::AccountId,
            kitty_id: u32,
            price: BalanceOf<T>,
            usd_price: Option<BalanceOf<T>>,/// Units 10^-10 usd cents duo to dot price ignore Balance decimal 12 
        },
        /// Event generated when new price is accepted to contribute to the average.
        NewPrice {
            price: u32,
            maybe_who: Option<T::AccountId>,
        },
    }
}
