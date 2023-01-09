#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::{OptionQuery, *};
	use frame_support::storage::hashed::put;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, ExistenceRequirement, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_std::vec::Vec;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u64,
		gender: Gender,
	}

	// Enum and implementation to handle Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		KittyDuplicate,
		KittyOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, Vec<u8>),
		KittyTransferred(T::AccountId, T::AccountId, Vec<u8>),
		SomethingStored(u32, T::AccountId),
	}

	#[pallet::storage]
	pub type Number<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	// Storage Kitty ID.
	#[pallet::storage]
	#[pallet::getter(fn kitty_id)]
	pub type KittyId<T> = StorageValue<_, u32, ValueQuery>;

	// Storage Kitties map.
	#[pallet::storage]
	#[pallet::getter(fn kitty_list)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	// Storage Kitties owned
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// // Put number
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		// pub fn put_number(origin: OriginFor<T>, number: u32) -> DispatchResult {
		// 	// check signed
		// 	let who = ensure_signed(origin)?;

		// 	// update storage
		// 	<Number<T>>::insert(who.clone(), number);

		// 	// emit event
		// 	Self::deposit_event(Event::KittyCreated(number, who));
		// 	Ok(())
		// }

		// Put number
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn put_number(origin: OriginFor<T>, number: u32) -> DispatchResult {
			// check signed
			let who = ensure_signed(origin)?;

			// update storage
			<Number<T>>::insert(who.clone(), number);

			// emit event
			Self::deposit_event(Event::SomethingStored(number, who));
			Ok(())
		}

		// create_kitty
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let owner = ensure_signed(origin)?;

			//check gender
			let gender = Self::generate_gender(dna.clone())?;

			let kitty: Kitty<T> =
				Kitty { dna: dna.clone(), owner: owner.clone(), price: 0, gender };

			// check kitty exist in storage or not ?
			ensure!(<Kitties<T>>::contains_key(kitty.dna.clone()), <Error<T>>::KittyDuplicate);

			// get index for new Kitty
			let current_id = <KittyId<T>>::get();
			let next_id = current_id.checked_add(1).ok_or(<Error<T>>::KittyOverflow)?;

			// Update storage.
			<KittyId<T>>::put(next_id);
			<Kitties<T>>::insert(kitty.dna.clone(), kitty.clone());
			<KittiesOwned<T>>::append(owner.clone(), kitty.dna.clone());

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(owner.clone(), dna));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// TODO Part III: set_price

		// TODO Part III: transfer

		// TODO Part III: buy_kitty

		// TODO Part III: breed_kitty
	}

	// TODO Parts II: helper function for Kitty struct

	impl<T: Config> Pallet<T> {
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from
	}
}

impl<T> Pallet<T> {
	fn generate_gender(dna: Vec<u8>) -> Result<Gender, Error<T>> {
		let mut res = Gender::Female;
		if dna.len() % 2 == 0 {
			res = Gender::Male;
		}
		Ok(res)
	}
}
