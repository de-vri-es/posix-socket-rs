use std::ffi::OsString;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

pub struct TempDir {
	path: PathBuf,
}

impl TempDir {
	pub fn new() -> std::io::Result<Self> {
		let mut template = std::env::temp_dir()
			.join("posix-socket-rs-XXXXXX")
			.into_os_string()
			.into_vec();
		template.push(0);
		unsafe {
			if libc::mkdtemp(template.as_mut_ptr() as *mut c_char).is_null() {
				return Err(std::io::Error::last_os_error());
			}
		}

		template.pop();
		let path = PathBuf::from(OsString::from_vec(template));
		Ok(Self { path })
	}

	pub fn path(&self) -> &Path {
		&self.path
	}
}

impl Drop for TempDir {
	fn drop(&mut self) {
		let _ = std::fs::remove_dir_all(self.path());
	}
}
