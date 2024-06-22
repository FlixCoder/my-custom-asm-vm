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
	/// Make a syscall to the given syscall index. The registers can be used
	/// to give arguments to the syscall, but handling differs across syscalls.
	Syscall(u8),
	/// Copy code memory to machine memory. Copies from the given source address
	/// to the address saved in the main register. Arguments: source, size.
	CopyCodeMemory(VmPtr, VmPtr),
	/// Data segment. Arguments: size/length, data.
	Data(VmPtr, Vec<u8>),
	/// Swap main register with register A.
	SwapRegisterA,
	/// Write the 8 bit value of register A to the address in the main register.
	Write8,
	/// Write the 16 bit value of register A to the address in the main
	/// register.
	Write16,
	/// Write the 32 bit value of register A to the address in the main
	/// register.
	Write32,
	/// Read stack pointer to main register.
	ReadStackPointer,
	/// Write main register to stack pointer.
	WriteStackPointer,
	/// Jump to given code address.
	Jump(VmPtr),
	/// Call function at given code address. Pushes the return address to the
	/// stack.
	Call(VmPtr),
	/// Return from function. This will pop the latest address from the stack
	/// and jump to it.
	Return,
	/// Increment the main register by one. Sets the zero flag to whether the
	/// result is 0.
	Increment,
	/// Decrement the main register by one. Sets the zero flag to whether the
	/// result is 0.
	Decrement,
	/// Main register += register A.
	Add,
	/// Main register -= register A.
	Sub,
	/// Compare main register with register A. Saves the comparison result in
	/// the comparison flag to be used in conditional jumps.
	Compare,
	/// Jump if the last comparison was equal.
	JumpEqual(VmPtr),
	/// Jump if the last comparison was not equal.
	JumpNotEqual(VmPtr),
	/// Jump if the last comparison was greater than.
	JumpGreater(VmPtr),
	/// Jump if the last comparison was less than.
	JumpLess(VmPtr),
	/// Jump if the last comparison was greater than or equal.
	JumpGreaterEqual(VmPtr),
	/// Jump if the last comparison was less than or equal.
	JumpLessEqual(VmPtr),
	/// Jump if the last increment/decrement resulted in zero.
	JumpZero(VmPtr),
	/// Jump if the last increment/decrement resulted in nonzero.
	JumpNonzero(VmPtr),
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
			Self::CopyCodeMemory(_, _) => 1 + 2 * size_of::<VmPtr>(),
			Self::Data(_len, data) => {
				assert_eq!(data.len(), native_ptr(*_len));
				1 + size_of::<VmPtr>() + data.len()
			}
			Self::SwapRegisterA => 1,
			Self::Write8 => 1,
			Self::Write16 => 1,
			Self::Write32 => 1,
			Self::ReadStackPointer => 1,
			Self::WriteStackPointer => 1,
			Self::Jump(_) => 1 + size_of::<VmPtr>(),
			Self::Call(_) => 1 + size_of::<VmPtr>(),
			Self::Return => 1,
			Self::Increment => 1,
			Self::Decrement => 1,
			Self::Add => 1,
			Self::Sub => 1,
			Self::Compare => 1,
			Self::JumpEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpNotEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpGreater(_) => 1 + size_of::<VmPtr>(),
			Self::JumpLess(_) => 1 + size_of::<VmPtr>(),
			Self::JumpGreaterEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpLessEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpZero(_) => 1 + size_of::<VmPtr>(),
			Self::JumpNonzero(_) => 1 + size_of::<VmPtr>(),
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
			)),
			14 => {
				let len = read_vm_ptr(code_sub_slice(1..)?)?;
				Ok(Self::Data(len, read_bytes(code_sub_slice(5..)?, native_ptr(len))?.to_vec()))
			}
			15 => Ok(Self::SwapRegisterA),
			16 => Ok(Self::Write8),
			17 => Ok(Self::Write16),
			18 => Ok(Self::Write32),
			19 => Ok(Self::ReadStackPointer),
			20 => Ok(Self::WriteStackPointer),
			21 => Ok(Self::Jump(read_vm_ptr(code_sub_slice(1..)?)?)),
			22 => Ok(Self::Call(read_vm_ptr(code_sub_slice(1..)?)?)),
			23 => Ok(Self::Return),
			24 => Ok(Self::Increment),
			25 => Ok(Self::Decrement),
			26 => Ok(Self::Add),
			27 => Ok(Self::Sub),
			28 => Ok(Self::Compare),
			29 => Ok(Self::JumpEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			30 => Ok(Self::JumpNotEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			31 => Ok(Self::JumpGreater(read_vm_ptr(code_sub_slice(1..)?)?)),
			32 => Ok(Self::JumpLess(read_vm_ptr(code_sub_slice(1..)?)?)),
			33 => Ok(Self::JumpGreaterEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			34 => Ok(Self::JumpLessEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			35 => Ok(Self::JumpZero(read_vm_ptr(code_sub_slice(1..)?)?)),
			36 => Ok(Self::JumpNonzero(read_vm_ptr(code_sub_slice(1..)?)?)),
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
			Self::CopyCodeMemory(src, size) => {
				bytes.push(13);
				bytes.extend_from_slice(&src.to_be_bytes());
				bytes.extend_from_slice(&size.to_be_bytes());
			}
			Self::Data(len, data) => {
				assert_eq!(data.len(), native_ptr(*len));
				bytes.push(14);
				bytes.extend_from_slice(&len.to_be_bytes());
				bytes.extend_from_slice(data);
			}
			Self::SwapRegisterA => bytes.push(15),
			Self::Write8 => bytes.push(16),
			Self::Write16 => bytes.push(17),
			Self::Write32 => bytes.push(18),
			Self::ReadStackPointer => bytes.push(19),
			Self::WriteStackPointer => bytes.push(20),
			Self::Jump(addr) => {
				bytes.push(21);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::Call(addr) => {
				bytes.push(22);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::Return => bytes.push(23),
			Self::Increment => bytes.push(24),
			Self::Decrement => bytes.push(25),
			Self::Add => bytes.push(26),
			Self::Sub => bytes.push(27),
			Self::Compare => bytes.push(28),
			Self::JumpEqual(addr) => {
				bytes.push(29);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpNotEqual(addr) => {
				bytes.push(30);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpGreater(addr) => {
				bytes.push(31);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpLess(addr) => {
				bytes.push(32);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpGreaterEqual(addr) => {
				bytes.push(33);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpLessEqual(addr) => {
				bytes.push(34);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpZero(addr) => {
				bytes.push(35);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
			Self::JumpNonzero(addr) => {
				bytes.push(36);
				bytes.extend_from_slice(&addr.to_be_bytes());
			}
		}
		bytes
	}
}
