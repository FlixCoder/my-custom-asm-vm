use std::mem::size_of;

use anyhow::Context;

use crate::{
	util::{native_ptr, read_bytes, read_u8, read_vm_ptr},
	VmPtr,
};

/// Instruction of my custom binary assembler language.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
	/// No instruction.
	Nop,
	/// Halt execution.
	Halt,
	/// Load 8 bit value from given address into the main register.
	Load8(VmPtr),
	/// Store 8 bit value into given address from the main register.
	Store8(VmPtr),
	/// Load 16 bit value from given address into the main register.
	Load16(VmPtr),
	/// Store 16 bit value into given address from the main register.
	Store16(VmPtr),
	/// Load 32 bit value from given address into the main register.
	Load32(VmPtr),
	/// Store 32 bit value into given address from the main register.
	Store32(VmPtr),
	/// Set main register to the given value.
	Set(VmPtr),
	/// Dereference the pointer in the main register to the 8 bit value it
	/// points to.
	Deref8,
	/// Dereference the pointer in the main register to the 16 bit value it
	/// points to.
	Deref16,
	/// Dereference the pointer in the main register to the 32 bit value it
	/// points to.
	Deref32,
	/// Make a syscall to the given syscall index. The main register can be used
	/// to give arguments to the syscall, but handling differs across syscalls.
	Syscall(u8),
	/// Copy code memory to usable memory. Arguments: source, target, size.
	CopyCodeMemory(VmPtr, VmPtr, VmPtr),
	/// Data segment. Arguments: size/length, data.
	Data(VmPtr, Vec<u8>),
}

impl Instruction {
	/// Return the length of bytes this instruction has.
	pub fn size(&self) -> usize {
		match self {
			Self::Nop => 1,
			Self::Halt => 1,
			Self::Load8(_) => 1 + size_of::<VmPtr>(),
			Self::Store8(_) => 1 + size_of::<VmPtr>(),
			Self::Load16(_) => 1 + size_of::<VmPtr>(),
			Self::Store16(_) => 1 + size_of::<VmPtr>(),
			Self::Load32(_) => 1 + size_of::<VmPtr>(),
			Self::Store32(_) => 1 + size_of::<VmPtr>(),
			Self::Set(_) => 1 + size_of::<VmPtr>(),
			Self::Deref8 => 1,
			Self::Deref16 => 1,
			Self::Deref32 => 1,
			Self::Syscall(_) => 2,
			Self::CopyCodeMemory(_, _, _) => 1 + 3 * size_of::<VmPtr>(),
			Self::Data(_len, data) => {
				assert_eq!(data.len(), native_ptr(*_len));
				1 + size_of::<VmPtr>() + data.len()
			}
		}
	}

	/// Parse the first instruction from the byte buffer.
	pub fn parse(code: &[u8]) -> anyhow::Result<Self> {
		let code_sub_slice = |index| code.get(index).context("not enough bytes");

		match *code.first().context("Cannot parse instruction from empty code")? {
			0 => Ok(Self::Nop),
			1 => Ok(Self::Halt),
			2 => Ok(Self::Load8(read_vm_ptr(code_sub_slice(1..)?)?)),
			3 => Ok(Self::Store8(read_vm_ptr(code_sub_slice(1..)?)?)),
			4 => Ok(Self::Load16(read_vm_ptr(code_sub_slice(1..)?)?)),
			5 => Ok(Self::Store16(read_vm_ptr(code_sub_slice(1..)?)?)),
			6 => Ok(Self::Load32(read_vm_ptr(code_sub_slice(1..)?)?)),
			7 => Ok(Self::Store32(read_vm_ptr(code_sub_slice(1..)?)?)),
			8 => Ok(Self::Set(read_vm_ptr(code_sub_slice(1..)?)?)),
			9 => Ok(Self::Deref8),
			10 => Ok(Self::Deref16),
			11 => Ok(Self::Deref32),
			12 => Ok(Self::Syscall(read_u8(code_sub_slice(1..)?)?)),
			13 => Ok(Self::CopyCodeMemory(
				read_vm_ptr(code_sub_slice(1..)?)?,
				read_vm_ptr(code_sub_slice(5..)?)?,
				read_vm_ptr(code_sub_slice(9..)?)?,
			)),
			14 => {
				let len = read_vm_ptr(code_sub_slice(1..)?)?;
				Ok(Self::Data(len, read_bytes(code_sub_slice(5..)?, native_ptr(len))?.to_vec()))
			}
			c => Err(anyhow::format_err!("Unrecognized instruction: {c}")),
		}
	}

	/// Convert this instruction to opcode bytes.
	pub fn bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(self.size());
		match self {
			Self::Nop => bytes.push(0),
			Self::Halt => bytes.push(1),
			Self::Load8(ptr) => {
				bytes.push(2);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Store8(ptr) => {
				bytes.push(3);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Load16(ptr) => {
				bytes.push(4);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Store16(ptr) => {
				bytes.push(5);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Load32(ptr) => {
				bytes.push(6);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Store32(ptr) => {
				bytes.push(7);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Set(ptr) => {
				bytes.push(8);
				bytes.extend_from_slice(&ptr.to_be_bytes());
			}
			Self::Deref8 => bytes.push(9),
			Self::Deref16 => bytes.push(10),
			Self::Deref32 => bytes.push(11),
			Self::Syscall(index) => {
				bytes.push(12);
				bytes.push(*index);
			}
			Self::CopyCodeMemory(src, target, size) => {
				bytes.push(13);
				bytes.extend_from_slice(&src.to_be_bytes());
				bytes.extend_from_slice(&target.to_be_bytes());
				bytes.extend_from_slice(&size.to_be_bytes());
			}
			Self::Data(len, data) => {
				assert_eq!(data.len(), native_ptr(*len));
				bytes.push(14);
				bytes.extend_from_slice(&len.to_be_bytes());
				bytes.extend_from_slice(data);
			}
		}
		bytes
	}
}
