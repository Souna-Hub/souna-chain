use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn test_create_kitty() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// current total kitty
		assert_eq!(KittiesModule::kitty_count(), 0);

		// check create new kitty
		assert_ok!(KittiesModule::create_kitty(Origin::signed(1)));
		assert_eq!(KittiesModule::kitty_count(), 1);

		// check kitty owned
		let kitty_owned = KittiesModule::kitty_owned(1);
		assert_eq!(kitty_owned.is_empty(), false);
	});
}

#[test]
fn test_transfer_kitty() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// create new kitty
		// current total kitty
		assert_eq!(KittiesModule::kitty_count(), 0);
		// check create new kitty extrinsis
		assert_ok!(KittiesModule::create_kitty(Origin::signed(1)));
		assert_eq!(KittiesModule::kitty_count(), 1);
		// check kitty owned
		let mut kitty_owned_from = KittiesModule::kitty_owned(1);
		assert_eq!(kitty_owned_from.is_empty(), false);

		//transfer
		// get kitty dna
		let kitty_dna = *kitty_owned_from.last().unwrap();
		let mut kitty_owned_to = KittiesModule::kitty_owned(2);
		assert_eq!(kitty_owned_to.is_empty(), true);

		// transfer extrinsis
		assert_ok!(KittiesModule::transfer_kitty(Origin::signed(1), 2, kitty_dna));

		// recheck kitty owned
		kitty_owned_to = KittiesModule::kitty_owned(2);
		kitty_owned_from = KittiesModule::kitty_owned(1);
		assert_eq!(kitty_owned_from.is_empty(), true);
		assert_eq!(kitty_owned_to.is_empty(), false);
	});
}

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(KittiesModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
// 	});
// }
