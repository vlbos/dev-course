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

            Kitties::<T>::insert(
                kitty_id,
                // Kitty( data.clone()),//Version 0
                Kitty {
                    dna: data.clone(),
                    price: None,
                },
            );
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
                    T::Currency::unreserve(&owner, T::StakeAmount::get());
                    <KittyOwner<T>>::insert(kitty_id, bidder.clone());
                    Self::deposit_event(Event::KittyTransferredAfterBidKnockedDown {
                        from: owner,
                        to: bidder,
                        kitty_id,
                        price,
                        usd_price: Self::average_price().map(|p| price * p.into()), //ignore Balance decimal 12    cents /dot 10^12
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

        /// Offchain Worker entry point.
        ///
        /// By implementing `fn offchain_worker` you declare a new offchain worker.
        /// This function will be called when the node is fully synced and a new best block is
        /// successfully imported.
        /// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
        /// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
        /// so the code should be able to handle that.
        /// You can use `Local Storage` API to coordinate runs of the worker.
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // Note that having logs compiled to WASM may cause the size of the blob to increase
            // significantly. You can use `RuntimeDebug` custom derive to hide details of the types
            // in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
            // all logging and thus, remove any logging from the WASM.
            log::info!("Kitties from offchain workers!");

            // Since off-chain workers are just part of the runtime code, they have direct access
            // to the storage and other included pallets.
            //
            // We can easily import `frame_system` and retrieve a block hash of the parent block.
            let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
            log::debug!(
                "Current block: {:?} (parent hash: {:?})",
                block_number,
                parent_hash
            );

            // It's a good practice to keep `fn offchain_worker()` function minimal, and move most
            // of the code to separate `impl` block.
            // Here we call a helper function to calculate current average price.
            // This function reads storage entries of the current state.
            let average: Option<u32> = Self::average_price();
            log::debug!("Current price: {:?}", average);

            // For this example we are going to send both signed and unsigned transactions
            // depending on the block number.
            // Usually it's enough to choose one or the other.
            let should_send = Self::choose_transaction_type(block_number);
            let res = match should_send {
                TransactionType::Signed => Self::fetch_price_and_send_signed(),
                TransactionType::UnsignedForAny => {
                    Self::fetch_price_and_send_unsigned_for_any_account(block_number)
                }
                TransactionType::UnsignedForAll => {
                    Self::fetch_price_and_send_unsigned_for_all_accounts(block_number)
                }
                TransactionType::Raw => Self::fetch_price_and_send_raw_unsigned(block_number),
                TransactionType::None => Ok(()),
            };
            if let Err(e) = res {
                log::error!("Error: {}", e);
            }
        }
        /// Chooses which transaction type to send.
        ///
        /// This function serves mostly to showcase `StorageValue` helper
        /// and local storage usage.
        ///
        /// Returns a type of transaction that should be produced in current run.
        fn choose_transaction_type(block_number: BlockNumberFor<T>) -> TransactionType {
            /// A friendlier name for the error that is going to be returned in case we are in the grace
            /// period.
            const RECENTLY_SENT: () = ();

            // Start off by creating a reference to Local Storage value.
            // Since the local storage is common for all offchain workers, it's a good practice
            // to prepend your entry with the module name.
            let val = StorageValueRef::persistent(b"example_ocw::last_send");
            // The Local Storage is persisted and shared between runs of the offchain workers,
            // and offchain workers may run concurrently. We can use the `mutate` function, to
            // write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
            // low-level method of local storage API, which means that only one worker
            // will be able to "acquire a lock" and send a transaction if multiple workers
            // happen to be executed concurrently.
            let res = val.mutate(
                |last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
                    match last_send {
                        // If we already have a value in storage and the block number is recent enough
                        // we avoid sending another transaction at this time.
                        Ok(Some(block)) if block_number < block + T::GracePeriod::get() => {
                            Err(RECENTLY_SENT)
                        }
                        // In every other case we attempt to acquire the lock and send a transaction.
                        _ => Ok(block_number),
                    }
                },
            );

            // The result of `mutate` call will give us a nested `Result` type.
            // The first one matches the return of the closure passed to `mutate`, i.e.
            // if we return `Err` from the closure, we get an `Err` here.
            // In case we return `Ok`, here we will have another (inner) `Result` that indicates
            // if the value has been set to the storage correctly - i.e. if it wasn't
            // written to in the meantime.
            match res {
                // The value has been set correctly, which means we can safely send a transaction now.
                Ok(block_number) => {
                    // We will send different transactions based on a random number.
                    // Note that this logic doesn't really guarantee that the transactions will be sent
                    // in an alternating fashion (i.e. fairly distributed). Depending on the execution
                    // order and lock acquisition, we may end up for instance sending two `Signed`
                    // transactions in a row. If a strict order is desired, it's better to use
                    // the storage entry for that. (for instance store both block number and a flag
                    // indicating the type of next transaction to send).
                    let transaction_type = block_number % 4u32.into();
                    if transaction_type == Zero::zero() {
                        TransactionType::Signed
                    } else if transaction_type == BlockNumberFor::<T>::from(1u32) {
                        TransactionType::UnsignedForAny
                    } else if transaction_type == BlockNumberFor::<T>::from(2u32) {
                        TransactionType::UnsignedForAll
                    } else {
                        TransactionType::Raw
                    }
                }
                // We are in the grace period, we should not send a transaction this time.
                Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => {
                    TransactionType::None
                }
                // We wanted to send a transaction, but failed to write the block number (acquire a
                // lock). This indicates that another offchain worker that was running concurrently
                // most likely executed the same logic and succeeded at writing to storage.
                // Thus we don't really want to send the transaction, knowing that the other run
                // already did.
                Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
            }
        }

        /// A helper function to fetch the price and send signed transaction.
        pub(super) fn fetch_price_and_send_signed() -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                );
            }
            // Make an external HTTP request to fetch the current price.
            // Note this call will block until response is received.
            let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

            // Using `send_signed_transaction` associated type we create and submit a transaction
            // representing the call, we've just created.
            // Submit signed will return a vector of results for all accounts that were found in the
            // local keystore with expected `KEY_TYPE`.
            let results = signer.send_signed_transaction(|_account| {
                // Received price is wrapped into a call to `submit_price` public function of this
                // pallet. This means that the transaction, when executed, will simply call that
                // function passing `price` as an argument.
                Call::submit_price { price }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted price of {} cents", acc.id, price),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }

        /// A helper function to fetch the price and send a raw unsigned transaction.
        pub(super) fn fetch_price_and_send_raw_unsigned(
            block_number: BlockNumberFor<T>,
        ) -> Result<(), &'static str> {
            // Make sure we don't fetch the price if unsigned transaction is going to be rejected
            // anyway.
            let next_unsigned_at = NextUnsignedAt::<T>::get();
            if next_unsigned_at > block_number {
                return Err("Too early to send unsigned transaction");
            }

            // Make an external HTTP request to fetch the current price.
            // Note this call will block until response is received.
            let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

            // Received price is wrapped into a call to `submit_price_unsigned` public function of this
            // pallet. This means that the transaction, when executed, will simply call that function
            // passing `price` as an argument.
            let call = Call::submit_price_unsigned {
                block_number,
                price,
            };

            // Now let's create a transaction out of this call and submit it to the pool.
            // Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
            //
            // By default unsigned transactions are disallowed, so we need to whitelist this case
            // by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefully
            // implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
            // attack vectors. See validation logic docs for more details.
            //
            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|()| "Unable to submit unsigned transaction.")?;

            Ok(())
        }

        /// A helper function to fetch the price, sign payload and send an unsigned transaction
        pub(super) fn fetch_price_and_send_unsigned_for_any_account(
            block_number: BlockNumberFor<T>,
        ) -> Result<(), &'static str> {
            // Make sure we don't fetch the price if unsigned transaction is going to be rejected
            // anyway.
            let next_unsigned_at = NextUnsignedAt::<T>::get();
            if next_unsigned_at > block_number {
                return Err("Too early to send unsigned transaction");
            }

            // Make an external HTTP request to fetch the current price.
            // Note this call will block until response is received.
            let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

            // -- Sign using any account
            let (_, result) = Signer::<T, T::AuthorityId>::any_account()
                .send_unsigned_transaction(
                    |account| PricePayload {
                        price,
                        block_number,
                        public: account.public.clone(),
                    },
                    |payload, signature| Call::submit_price_unsigned_with_signed_payload {
                        price_payload: payload,
                        signature,
                    },
                )
                .ok_or("No local accounts accounts available.")?;
            result.map_err(|()| "Unable to submit transaction")?;

            Ok(())
        }

        /// A helper function to fetch the price, sign payload and send an unsigned transaction
        pub(super) fn fetch_price_and_send_unsigned_for_all_accounts(
            block_number: BlockNumberFor<T>,
        ) -> Result<(), &'static str> {
            // Make sure we don't fetch the price if unsigned transaction is going to be rejected
            // anyway.
            let next_unsigned_at = NextUnsignedAt::<T>::get();
            if next_unsigned_at > block_number {
                return Err("Too early to send unsigned transaction");
            }

            // Make an external HTTP request to fetch the current price.
            // Note this call will block until response is received.
            let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

            // -- Sign using all accounts
            let transaction_results = Signer::<T, T::AuthorityId>::all_accounts()
                .send_unsigned_transaction(
                    |account| PricePayload {
                        price,
                        block_number,
                        public: account.public.clone(),
                    },
                    |payload, signature| Call::submit_price_unsigned_with_signed_payload {
                        price_payload: payload,
                        signature,
                    },
                );
            for (_account_id, result) in transaction_results.into_iter() {
                if result.is_err() {
                    return Err("Unable to submit transaction");
                }
            }

            Ok(())
        }

        /// Fetch current price and return the result in cents.
        pub(super) fn fetch_price() -> Result<u32, http::Error> {
            // We want to keep the offchain worker execution time reasonable, so we set a hard-coded
            // deadline to 2s to complete the external call.
            // You can also wait indefinitely for the response, however you may still get a timeout
            // coming from the host machine.
            let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
            // Initiate an external HTTP GET request.
            // This is using high-level wrappers from `sp_runtime`, for the low-level calls that
            // you can find in `sp_io`. The API is trying to be similar to `request`, but
            // since we are running in a custom WASM execution environment we can't simply
            // import the library here.
            let request = http::Request::get(
                "https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD",
            );
            // We set the deadline for sending of the request, note that awaiting response can
            // have a separate deadline. Next we send the request, before that it's also possible
            // to alter request headers or stream body content in case of non-GET requests.
            let pending = request
                .deadline(deadline)
                .send()
                .map_err(|_| http::Error::IoError)?;

            // The request is already being processed by the host, we are free to do anything
            // else in the worker (we can send multiple concurrent requests too).
            // At some point however we probably want to check the response though,
            // so we can block current thread and wait for it to finish.
            // Note that since the request is being driven by the host, we don't have to wait
            // for the request to have it complete, we will just not read the response.
            let response = pending
                .try_wait(deadline)
                .map_err(|_| http::Error::DeadlineReached)??;
            // Let's check the status code before we proceed to reading the response.
            if response.code != 200 {
                log::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown);
            }

            // Next we want to fully read the response body and collect it to a vector of bytes.
            // Note that the return object allows you to read the body in chunks as well
            // with a way to control the deadline.
            let body = response.body().collect::<Vec<u8>>();

            // Create a str slice from the body.
            let body_str = alloc::str::from_utf8(&body).map_err(|_| {
                log::warn!("No UTF8 body");
                http::Error::Unknown
            })?;

            let price = match Self::parse_price(body_str) {
                Some(price) => Ok(price),
                None => {
                    log::warn!("Unable to extract price from the response: {:?}", body_str);
                    Err(http::Error::Unknown)
                }
            }?;

            log::warn!("Got price: {} cents", price);

            Ok(price)
        }

        /// Parse the price from the given JSON string using `lite-json`.
        ///
        /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
        pub(super) fn parse_price(price_str: &str) -> Option<u32> {
            let val = lite_json::parse_json(price_str);
            let price = match val.ok()? {
                JsonValue::Object(obj) => {
                    let (_, v) = obj
                        .into_iter()
                        .find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
                    match v {
                        JsonValue::Number(number) => number,
                        _ => return None,
                    }
                }
                _ => return None,
            };

            let exp = price.fraction_length.saturating_sub(2);
            Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
        }

        /// Add new price to the list.
        fn add_price(maybe_who: Option<T::AccountId>, price: u32) {
            log::info!("Adding to the average: {}", price);
            <Prices<T>>::mutate(|prices| {
                if prices.try_push(price).is_err() {
                    prices[(price % T::MaxPrices::get()) as usize] = price;
                }
            });

            let average = Self::average_price()
                .expect("The average is not empty, because it was just mutated; qed");
            log::info!("Current average price is: {}", average);
            // here we are raising the NewPrice event
            Self::deposit_event(Event::NewPrice { price, maybe_who });
        }

        /// Calculate current average price.
        pub(super) fn average_price() -> Option<u32> {
            let prices = Prices::<T>::get();
            prices
                .clone()
                .into_iter()
                .reduce(|a, b| a.saturating_add(b))
                .map(|s| s / prices.len() as u32)
        }

        fn validate_transaction_parameters(
            block_number: &BlockNumberFor<T>,
            new_price: &u32,
        ) -> TransactionValidity {
            // Now let's check if the transaction has any chance to succeed.
            let next_unsigned_at = NextUnsignedAt::<T>::get();
            if &next_unsigned_at > block_number {
                return InvalidTransaction::Stale.into();
            }
            // Let's make sure to reject transactions from the future.
            let current_block = <system::Pallet<T>>::block_number();
            if &current_block < block_number {
                return InvalidTransaction::Future.into();
            }

            // We prioritize transactions that are more far away from current average.
            //
            // Note this doesn't make much sense when building an actual oracle, but this example
            // is here mostly to show off offchain workers capabilities, not about building an
            // oracle.
            let avg_price = Self::average_price()
                .map(|price| {
                    if &price > new_price {
                        price - new_price
                    } else {
                        new_price - price
                    }
                })
                .unwrap_or(0);

            ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
                // We set base priority to 2**20 and hope it's included before any other
                // transactions in the pool. Next we tweak the priority depending on how much
                // it differs from the current average. (the more it differs the more priority it
                // has).
                .priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
                // This transaction does not require anything else to go before into the pool.
                // In theory we could require `previous_unsigned_at` transaction to go first,
                // but it's not necessary in our case.
                //.and_requires()
                // We set the `provides` tag to be the same as `next_unsigned_at`. This makes
                // sure only one transaction produced after `next_unsigned_at` will ever
                // get to the transaction pool and will end up in the block.
                // We can still have multiple transactions compete for the same "spot",
                // and the one with higher priority will replace other one in the pool.
                .and_provides(next_unsigned_at)
                // The transaction is only valid for next 5 blocks. After that it's
                // going to be revalidated by the pool.
                .longevity(5)
                // It's fine to propagate that transaction to other peers, which means it can be
                // created even by nodes that don't produce blocks.
                // Note that sometimes it's better to keep it for yourself (if you are the block
                // producer), since for instance in some schemes others may copy your solution and
                // claim a reward.
                .propagate(true)
                .build()
        }
    }
}
