mod util;

use assert2::assert;

#[test]
fn test_temp_dir() {
	let temp_dir = util::TempDir::new().unwrap();
	let dir = temp_dir.path().to_owned();
	assert!(dir.is_dir());

	let file = dir.join("foo");
	std::fs::write(&file, "Hello!").unwrap();
	assert!(file.is_file());

	drop(temp_dir);
	assert!(!dir.is_dir());
}
