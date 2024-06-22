use std::ffi::CStr;

use anyhow::Context;

use crate::VmPtr;

/// Get a native pointer from a VmPtr.
pub fn native_ptr(ptr: VmPtr) -> usize {
	ptr.try_into().expect("VmPtr cannot be usize")
}

/// Get a VmPtr from a native pointer.
pub fn vm_ptr(ptr: usize) -> VmPtr {
	ptr.try_into().expect("usize cannot be VmPtr")
}

/// Read the first bytes from a buffer and convert it to a u8.
pub fn read_u8(bytes: &[u8]) -> anyhow::Result<u8> {
	bytes.first().context("not enough bytes").copied()
}

/// Write an u8 to a buffer.
pub fn write_u8(buffer: &mut [u8], value: u8) -> anyhow::Result<()> {
	*buffer.first_mut().context("not enough bytes")? = value;
	Ok(())
}

/// Read the first bytes from a buffer and convert it to a u16.
pub fn read_u16(bytes: &[u8]) -> anyhow::Result<u16> {
	let bytes =
		[*bytes.first().context("not enough bytes")?, *bytes.get(1).context("not enough bytes")?];
	Ok(u16::from_be_bytes(bytes))
}

/// Write an u16 to a buffer.
pub fn write_u16(buffer: &mut [u8], value: u16) -> anyhow::Result<()> {
	let bytes = value.to_be_bytes();
	*buffer.first_mut().context("not enough bytes")? = bytes[0];
	*buffer.get_mut(1).context("not enough bytes")? = bytes[1];
	Ok(())
}

/// Read the first bytes from a buffer and convert it to a u32.
pub fn read_u32(bytes: &[u8]) -> anyhow::Result<u32> {
	let bytes = [
		*bytes.first().context("not enough bytes")?,
		*bytes.get(1).context("not enough bytes")?,
		*bytes.get(2).context("not enough bytes")?,
		*bytes.get(3).context("not enough bytes")?,
	];
	Ok(u32::from_be_bytes(bytes))
}

/// Write an u32 to a buffer.
pub fn write_u32(buffer: &mut [u8], value: u32) -> anyhow::Result<()> {
	let bytes = value.to_be_bytes();
	*buffer.first_mut().context("not enough bytes")? = bytes[0];
	*buffer.get_mut(1).context("not enough bytes")? = bytes[1];
	*buffer.get_mut(2).context("not enough bytes")? = bytes[2];
	*buffer.get_mut(3).context("not enough bytes")? = bytes[3];
	Ok(())
}

/// Read the first bytes from a buffer and convert it to a VmPtr.
pub fn read_vm_ptr(bytes: &[u8]) -> anyhow::Result<VmPtr> {
	read_u32(bytes)
}

/// Write a VmPtr to a buffer.
#[allow(dead_code)] // Currently unneded, but whatever.
pub fn write_vm_ptr(buffer: &mut [u8], value: VmPtr) -> anyhow::Result<()> {
	write_u32(buffer, value)
}

/// Read the given amount of bytes from a buffer.
pub fn read_bytes(buffer: &[u8], len: usize) -> anyhow::Result<&[u8]> {
	buffer.get(0..len).context("not enough bytes")
}

/// Read a CStr from a buffer.
pub fn read_cstr(buffer: &[u8]) -> anyhow::Result<&CStr> {
	CStr::from_bytes_until_nul(buffer).context("not enough bytes")
}
