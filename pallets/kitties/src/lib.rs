#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::traits::Randomness;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::fmt;
	use frame_support::{
		pallet_prelude::*,
		traits::{Randomness, Time},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: T::Hash,
		pub owner: T::AccountId,
		pub price: u64,
		pub image_url: Vec<u8>,
		pub gender: Gender,
		pub created_date: <<T as Config>::Time as Time>::Moment,
	}

	// Implement debug for kitty
	impl<T: Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
				.field("dna", &self.dna)
				.field("owner", &self.owner)
				.field("price", &self.price)
				.field("gender", &self.gender)
				.field("created_date", &self.created_date)
				.finish()
		}
	}

	// Enum and implementation to handle Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Time: Time;
		type KittyRandomDna: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type KittyOwnedLimit: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type Number<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	// Storage total kitty.
	#[pallet::storage]
	#[pallet::getter(fn kitty_count)]
	pub type CountKitty<T> = StorageValue<_, u32, ValueQuery>;

	// Storage Kitties map.
	#[pallet::storage]
	#[pallet::getter(fn kitty_list)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>>;

	// Storage Kitties owned
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub type KittiesOwned<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::KittyOwnedLimit>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		KittyCreated(T::AccountId, T::Hash),
		KittyTransferred(T::AccountId, T::AccountId, T::Hash),
		KittyDefaultListGenerated(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		KittyDuplicate,
		KittyOverflow,
		NoneKitty,
		OverKittyOwnedLimit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		// create_kitty
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let owner = ensure_signed(origin)?;

			// Generate dna for kitty
			let dna = Self::generate_dna()?;

			// Check gender
			let gender = Self::generate_gender(dna.clone())?;

			// Get create date
			let created_date = T::Time::now();

			// image kitty
			let image_url_origin = "https://img.cryptokitties.co/0x06012c8cf97bead5deae237070f9587f8e7a266d/1450983.png";
			let image_url = image_url_origin.chars().map(|c| c as u8).collect::<Vec<_>>();
			// Create new kitty instance
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				owner: owner.clone(),
				price: 0,
				gender,
				image_url,
				created_date,
			};

			// Check kitty overflow
			let current_id = <CountKitty<T>>::get();
			let next_id = current_id.checked_add(1).ok_or(<Error<T>>::KittyOverflow)?;

			// Update kitties owned
			let mut kitties_owned = KittiesOwned::<T>::get(&owner);
			kitties_owned
				.try_push(dna.clone())
				.map_err(|_| <Error<T>>::OverKittyOwnedLimit)?;

			// Update storage
			<KittiesOwned<T>>::insert(&owner, kitties_owned);
			<CountKitty<T>>::put(next_id);
			<Kitties<T>>::insert(dna.clone(), kitty);

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(owner.clone(), dna.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_kitty(
			origin: OriginFor<T>,
			to: T::AccountId,
			dna: T::Hash,
		) -> DispatchResult {
			// Check signed
			let from = ensure_signed(origin)?;

			let mut kitty = <Kitties<T>>::get(dna.clone()).ok_or(<Error<T>>::NoneKitty)?;
			let mut from_owned = KittiesOwned::<T>::get(&from);

			// Remove kitty from list of owned kitties.
			if let Some(ind) = from_owned.iter().position(|ids| *ids == dna) {
				from_owned.swap_remove(ind);
			} else {
				return Err(<Error<T>>::NoneKitty.into());
			}

			// Update new owner for kitty
			let mut to_owned = KittiesOwned::<T>::get(&to);
			to_owned.try_push(dna.clone()).map_err(|_| <Error<T>>::OverKittyOwnedLimit).ok();
			kitty.owner = to.clone();

			// Write updates to storage
			<Kitties<T>>::insert(&dna, kitty);
			<KittiesOwned<T>>::insert(&to, to_owned);
			<KittiesOwned<T>>::insert(&from, from_owned);

			// Emit an event.
			Self::deposit_event(Event::KittyTransferred(from.clone(), to.clone(), dna.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn generate_default_kitty_list(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/

			let mut kittyDefautUrl = "https://img.cryptokitties.co/0x06012c8cf97bead5deae237070f9587f8e7a266d/1450983.png".chars().map(|c| c as u8).collect::<Vec<_>>();

			let owner = ensure_signed(origin)?;

			for i in 0..10 {
				// Generate dna for kitty
				let dna = Self::generate_dna()?;

				// Check gender
				let gender = Self::generate_gender(dna.clone())?;

				// Get create date
				let created_date = T::Time::now();

				// Create new kitty instance
				let kitty = Kitty::<T> {
					dna: dna.clone(),
					owner: owner.clone(),
					price: 0,
					image_url: kittyDefautUrl.clone(),
					gender,
					created_date,
				};

				// Update kitties owned
				let mut kitties_owned = KittiesOwned::<T>::get(&owner);
				kitties_owned
					.try_push(dna.clone())
					.map_err(|_| <Error<T>>::OverKittyOwnedLimit)?;

				<KittiesOwned<T>>::insert(&owner, kitties_owned);
				<Kitties<T>>::insert(dna.clone(), kitty);
			}

			// Update storage
			<CountKitty<T>>::put(10);

			// Emit an event.
			Self::deposit_event(Event::KittyDefaultListGenerated(owner.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Generate random DNA for Kitty
	fn generate_dna() -> Result<T::Hash, Error<T>> {
		let mut res = T::KittyRandomDna::random(&b"kittydna"[..]).0;
		while Self::kitty_list(res) != None {
			res = T::KittyRandomDna::random(&b"kittydna"[..]).0;
		}
		Ok(res)
	}

	// Generate random Gender for Kitty
	fn generate_gender(dna: T::Hash) -> Result<Gender, Error<T>> {
		let mut res = Gender::Female;
		if dna.encode()[0] % 2 == 0 {
			res = Gender::Male;
		}
		Ok(res)
	}
}
