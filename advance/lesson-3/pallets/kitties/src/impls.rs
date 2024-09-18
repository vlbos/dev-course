use frame_support::pallet_macros::pallet_section;

/// Define the implementation of the pallet, like helper functions.
#[pallet_section]
mod impls {
    impl<T: Config> Pallet<T> {
        // get a random 256.
        fn random_value(who: &T::AccountId) -> [u8; 16] {
            let nonce = frame_system::Pallet::<T>::account_nonce(&who);
            // let nonce_u32: u32 = nonce as u32;
            // generate a random value based on account and its nonce
            let nonce_u32: u32 = TryInto::try_into(nonce).ok().expect("nonce is u64; qed");
            let a: BlockNumberFor<T> = TryFrom::try_from(nonce_u32)
                .ok()
                .expect("nonce is u32; qed");
            (
                T::Randomness::random_seed(),
                a,
                <frame_system::Pallet<T>>::extrinsic_index(),
            )
                .using_encoded(sp_io::hashing::blake2_128)
        }

        fn mint_kitty(who: &T::AccountId, data: [u8; 16]) -> DispatchResult {
            let kitty_id = Self::next_kitty_id()
                .checked_add(1)
                .ok_or(Error::<T>::NextKittyIdOverflow)?;

            let stake_amount = T::StakeAmount::get();

            T::Currency::reserve(&who, stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

            Kitties::<T>::insert(kitty_id, Kitty(data.clone()));
            KittyOwner::<T>::insert(kitty_id, who.clone());
            NextKittyId::<T>::put(kitty_id);

            Self::deposit_event(Event::KittyCreated {
                creator: who.clone(),
                kitty_id,
                data,
            });

            Ok(())
        }

        // breed on kitty based on both parent kitties
        fn breed_kitty(who: &T::AccountId, kitty_1: [u8; 16], kitty_2: [u8; 16]) -> [u8; 16] {
            use core::convert::TryInto;
            kitty_1
                .into_iter()
                .zip(kitty_2)
                .zip(Self::random_value(who))
                .map(|((k1, k2), s)| (k1 & s) | (k2 & !s))
                .collect::<Vec<_>>()
                .try_into()
                .expect("convert Vec<u8> to [u8; 16] failed")
        }
        fn transfer_kitty(from: T::AccountId, to: T::AccountId, kitty_id: u32) -> DispatchResult {
            ensure!(from != to, Error::<T>::TransferToSelf);

            ensure!(
                Self::kitty_owner(kitty_id).as_ref() == Some(&from),
                Error::<T>::NotOwner
            );

            let stake_amount = T::StakeAmount::get();

            T::Currency::reserve(&to, stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;
            T::Currency::unreserve(&from, stake_amount);

            <KittyOwner<T>>::insert(kitty_id, to.clone());

            Self::deposit_event(Event::KittyTransferred { from, to, kitty_id });

            Ok(())
        }

        fn trade(until_block: BlockNumberFor<T>) -> DispatchResult {
            let bids = KittiesOnSale::<T>::take(until_block);

            for kitty_id in bids {
                let Some((bidder, price)) = KittiesBid::<T>::take(kitty_id) else {
                    log::warn!(
                        "Kitties bid unsold  at block {:?}, {:?},{}",
                        until_block,
                        Self::kitty_owner(kitty_id),
                        kitty_id
                    );
                    continue;
                };
                let owner = Self::kitty_owner(kitty_id).expect("Invalid kitty id");
                if T::Currency::reserved_balance(&bidder) < price {
                    log::warn!(
                            "Unexpected Kitties bid Currency::reserved_balacne less than price at block {:?}, {:?},{:?},{}",
                            until_block,
                            owner,
                            bidder,
                            kitty_id
                        );
                }
                let actual_unreserve_balance = T::Currency::unreserve(&bidder, price);
                if actual_unreserve_balance < price {
                    log::warn!(
                                "Unexpected Kitties bid Currency::unreserve less than price  at block {:?}, {:?},{:?},{}",
                                until_block,
                                owner,
                                bidder,
                                kitty_id
                            );
                }
                if T::Currency::free_balance(&bidder) < price {
                    log::warn!(
                                "Unexpected Kitties bid free_balance less than price  at block {:?}, {:?},{:?},{}",
                                until_block,
                                owner,
                                bidder,
                                kitty_id
                            );
                }

                if T::Currency::transfer(
                    &bidder,
                    &owner,
                    price,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )
                .is_ok()
                {
                    log::info!(
                        "Kitties bid Currency::transfer  at block {:?}, {:?},{:?},{},",
                        until_block,
                        owner,
                        bidder,
                        kitty_id
                    );
                    <KittyOwner<T>>::insert(kitty_id, bidder.clone());
                    Self::deposit_event(Event::KittyTransferred {
                        from: owner,
                        to: bidder,
                        kitty_id,
                    });
                } else {
                    log::warn!(
                        "Kitties bid Currency::transfer failed at block {:?},{:?},{:?},{}",
                        until_block,
                        owner,
                        bidder,
                        kitty_id
                    );
                }
            }

            Ok(())
        }
    }
}
