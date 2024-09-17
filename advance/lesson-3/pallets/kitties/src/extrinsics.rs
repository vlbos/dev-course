use frame_support::pallet_macros::pallet_section;

/// Define all extrinsics for the pallet.
#[pallet_section]
mod dispatches {
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create())]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = Self::random_value(&who);
            Self::mint_kitty(&who, value)?;
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::breed())]
        pub fn breed(origin: OriginFor<T>, kitty_1: u32, kitty_2: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(kitty_1 != kitty_2, Error::<T>::SameParentId);
            let (Kitty(kitty_1), Kitty(kitty_2)) = (
                Self::kitties(kitty_1).ok_or(Error::<T>::KittyNotExist)?,
                Self::kitties(kitty_2).ok_or(Error::<T>::KittyNotExist)?,
            );
            let value = Self::breed_kitty(&who, kitty_1, kitty_2);
            Self::mint_kitty(&who, value)?;
            Ok(())
        }
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !KittiesBid::<T>::contains_key(kitty_id),
                Error::<T>::KittyAlreadyOnSale
            );
            Self::transfer_kitty(who, to, kitty_id)?;
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::sale())]
        pub fn sale(
            origin: OriginFor<T>,
            kitty_id: u32,
            until_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::kitty_owner(kitty_id).as_ref() == Some(&who),
                Error::<T>::NotOwner
            );
            ensure!(
                !KittiesBid::<T>::contains_key(kitty_id),
                Error::<T>::KittyAlreadyOnSale
            );
            ensure!(
                until_block >= <system::Pallet<T>>::block_number() + T::MinBidBlockSpan::get(),
                Error::<T>::BlockSpanTooSmall
            );

            KittiesOnSale::<T>::try_append(&until_block, kitty_id)
                .map_err(|_| Error::<T>::TooManyBidOnOneBlock)?;
            KittiesBid::<T>::insert(kitty_id, Option::<(T::AccountId, BalanceOf<T>)>::default());
            Self::deposit_event(Event::KittyOnSale {
                owner: who,
                kitty_id,
                until_block,
            });

            Ok(())
        }
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::bid())]
        pub fn bid(origin: OriginFor<T>, kitty_id: u32, price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(&who) != Self::kitty_owner(kitty_id).as_ref(),
                Error::<T>::BidForSelf
            );
            let last_bid =
                KittiesBid::<T>::try_get(kitty_id).map_err(|_| Error::<T>::KittyNotOnSale)?;
            let stake_amount = T::StakeAmount::get();
            if let Some((last_bidder, last_price)) = last_bid {
                ensure!(
                    price >= last_price + T::MinBidIncrement::get(),
                    Error::<T>::KittyBidLessThanTheSumOfLastPriceAndMinimumBidIncrement
                );
                T::Currency::unreserve(&last_bidder, last_price + stake_amount);
            } else {
                ensure!(
                    price >= T::MinBidAmount::get(),
                    Error::<T>::KittyBidLessThanOrMinimumBidAmount
                );
            }

            T::Currency::reserve(&who, price + stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalanceForBidAndStaking)?;
            KittiesBid::<T>::insert(kitty_id, Some((who.clone(), price)));
            Self::deposit_event(Event::KittyBid {
                bidder: who,
                kitty_id,
                price,
            });
            Ok(())
        }
    }
}
