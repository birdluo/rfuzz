extern crate libc;

use std;

pub struct SharedMemory {
	data: *mut u8,
	size: usize,
	id: i32,
	write_offset: usize,
}

impl SharedMemory {
	pub fn create(size: usize) -> SharedMemory {
		let shm_id = unsafe {
			libc::shmget(libc::IPC_PRIVATE, size,
						 libc::IPC_CREAT | libc::IPC_EXCL | 0o600)
			};
		assert!(shm_id >= 0);
		let data = unsafe { libc::shmat(shm_id, std::ptr::null_mut(), 0) };
		if data as i64 == -1 {
			panic!("failed to allocate shared memory: {:?}", std::io::Error::last_os_error());
		}
		// println!("SharedMemory.create: id={}, data={:?}", shm_id, data);
		let ptr = data as *mut u8;
		SharedMemory { id: shm_id, size: size, data: ptr, write_offset: 0 }
	}

	pub fn reset_write_offset(&mut self) {
		self.write_offset = 0;
	}

	pub fn reset(&mut self) {
		self.write_offset = 0;
		unsafe { libc::memset(self.data as *mut libc::c_void, 0, self.size) };
	}

	pub fn as_slice_u32_mut(&mut self) -> &mut [u32] {
		unsafe { std::slice::from_raw_parts_mut(self.data as *mut u32, self.size / 4) }
	}

	pub fn bytes_left(&self) -> usize {
		self.size - self.write_offset
	}

	pub fn write_all(&mut self, buf: &[u8]) -> Result<(), ()> {
		if buf.len() > self.bytes_left() { Err(()) }
		else {
			let len = buf.len();
			let dst = unsafe{ self.data.offset(self.write_offset as isize) } as *mut libc::c_void;
			let src = buf.as_ptr() as *const libc::c_void;
			unsafe { libc::memcpy(dst, src, len) };
			self.write_offset += len;
			Ok(())
		}
	}

	pub fn write_zeros(&mut self, len: usize) -> Result<(), ()> {
		if len > self.bytes_left() { Err(()) }
		else {
			let dst = unsafe{ self.data.offset(self.write_offset as isize) } as *mut libc::c_void;
			unsafe { libc::memset(dst, 0, len) };
			self.write_offset += len;
			Ok(())
		}
	}

	pub fn write_u32(&mut self, val: u32) -> Result<(), ()> {
		let data = [(val >>  0) as u8, (val >>  8) as u8,
		            (val >> 16) as u8, (val >> 24) as u8];
		self.write_all(&data)
	}

	pub fn write_u64(&mut self, val: u64) -> Result<(), ()> {
		let data = [(val >>  0) as u8, (val >>  8) as u8,
		            (val >> 16) as u8, (val >> 24) as u8,
		            (val >> 32) as u8, (val >> 40) as u8,
		            (val >> 48) as u8, (val >> 56) as u8];
		self.write_all(&data)
	}

	pub fn id(&self) -> i32 { self.id }
}

impl Drop for SharedMemory {
	fn drop(&mut self) {
		// println!("SharedMemory.drop: {}", self.id);
		unsafe { libc::shmctl(self.id, libc::IPC_RMID, std::ptr::null_mut()) };
	}
}
