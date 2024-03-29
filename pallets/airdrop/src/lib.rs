#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::sp_runtime::SaturatedConversion;
	use frame_support::traits::Currency;
	use frame_support::weights::{GetDispatchInfo, Pays};
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type LocalCurrency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type LastAirdrop<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [amount, who]
		Airdrop(u64, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		DelayNotFinished,
		SomethingWentWrong,
	}

	const DELAY: u32 = 16;

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1), DispatchClass::Normal, Pays::No))]
		pub fn get_tokens(origin: OriginFor<T>, amount: u64) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let last_user_airdrop_block =
				<LastAirdrop<T>>::get(&who).unwrap_or_else(|| T::BlockNumber::from(0u32));

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			if current_block_number - last_user_airdrop_block < T::BlockNumber::from(DELAY) {
				return Err(Error::<T>::DelayNotFinished.into());
			}

			let balance = amount.saturated_into();

			let imb = T::LocalCurrency::issue(balance);

			if T::LocalCurrency::resolve_into_existing(&who, imb).is_err() {
				return Err(Error::<T>::SomethingWentWrong.into());
			};

			<LastAirdrop<T>>::insert(&who, current_block_number);

			Self::deposit_event(Event::Airdrop(amount, who));

			Ok(())
		}
	}
}
