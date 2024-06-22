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
	/// Dereference the pointer in the register x to the 8 bit value it
	/// points to and write the result to the main register.
	Deref8(u8),
	/// Dereference the pointer in the register x to the 16 bit value it
	/// points to and write the result to the main register.
	Deref16(u8),
	/// Dereference the pointer in the register x to the 32 bit value it
	/// points to and write the result to the main register.
	Deref32(u8),
	/// Make a syscall to the given syscall index. The registers can be used
	/// to give arguments to the syscall, but handling differs across syscalls.
	Syscall(u8),
	/// Copy code memory to machine memory. Copies from the given source address
	/// to the address saved in the main register. Arguments: source, size.
	CopyCodeMemory(VmPtr, VmPtr),
	/// Data segment. Arguments: size/length, data.
	Data(VmPtr, Vec<u8>),
	/// Swap main register with given side register.
	Swap(u8),
	/// Write the 8 bit value of the main register to the address in register x.
	Write8(u8),
	/// Write the 16 bit value of the main register to the address in register
	/// x.
	Write16(u8),
	/// Write the 32 bit value of the main register to the address in register
	/// x.
	Write32(u8),
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
	/// Main register += register x.
	Add(u8),
	/// Main register -= register x.
	Sub(u8),
	/// Compare main register with register x. Saves the comparison result in
	/// the comparison flag to be used in conditional jumps.
	Compare(u8),
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
	/// Push main register to the stack.
	Push,
	/// Pop from the stack to the main register.
	Pop,
	/// Push register x to the stack.
	PushRegister(u8),
	/// Pop from the stack to register x.
	PopRegister(u8),
	/// Multiplication of the main register by register x. The result is saved
	/// in the main register.
	Mul(u8),
	/// Division of the main register by register x. The result is saved in the
	/// main register, the remainder in register x.
	Div(u8),
	/// Increment the given side register.
	IncrementRegister(u8),
	/// Decrement the given side register.
	DecrementRegister(u8),
	/// Set a side register to a specific value.
	SetRegister(u8, VmPtr),
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
			Self::Deref8(_) => 2,
			Self::Deref16(_) => 2,
			Self::Deref32(_) => 2,
			Self::Syscall(_) => 2,
			Self::CopyCodeMemory(_, _) => 1 + 2 * size_of::<VmPtr>(),
			Self::Data(_len, data) => {
				assert_eq!(data.len(), native_ptr(*_len));
				1 + size_of::<VmPtr>() + data.len()
			}
			Self::Swap(_) => 2,
			Self::Write8(_) => 2,
			Self::Write16(_) => 2,
			Self::Write32(_) => 2,
			Self::ReadStackPointer => 1,
			Self::WriteStackPointer => 1,
			Self::Jump(_) => 1 + size_of::<VmPtr>(),
			Self::Call(_) => 1 + size_of::<VmPtr>(),
			Self::Return => 1,
			Self::Increment => 1,
			Self::Decrement => 1,
			Self::Add(_) => 2,
			Self::Sub(_) => 2,
			Self::Compare(_) => 2,
			Self::JumpEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpNotEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpGreater(_) => 1 + size_of::<VmPtr>(),
			Self::JumpLess(_) => 1 + size_of::<VmPtr>(),
			Self::JumpGreaterEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpLessEqual(_) => 1 + size_of::<VmPtr>(),
			Self::JumpZero(_) => 1 + size_of::<VmPtr>(),
			Self::JumpNonzero(_) => 1 + size_of::<VmPtr>(),
			Self::Push => 1,
			Self::Pop => 1,
			Self::PushRegister(_) => 2,
			Self::PopRegister(_) => 2,
			Self::Mul(_) => 2,
			Self::Div(_) => 2,
			Self::IncrementRegister(_) => 2,
			Self::DecrementRegister(_) => 2,
			Self::SetRegister(_, _) => 2 + size_of::<VmPtr>(),
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
			9 => Ok(Self::Deref8(read_u8(code_sub_slice(1..)?)?)),
			10 => Ok(Self::Deref16(read_u8(code_sub_slice(1..)?)?)),
			11 => Ok(Self::Deref32(read_u8(code_sub_slice(1..)?)?)),
			12 => Ok(Self::Syscall(read_u8(code_sub_slice(1..)?)?)),
			13 => Ok(Self::CopyCodeMemory(
				read_vm_ptr(code_sub_slice(1..)?)?,
				read_vm_ptr(code_sub_slice(5..)?)?,
			)),
			14 => {
				let len = read_vm_ptr(code_sub_slice(1..)?)?;
				Ok(Self::Data(len, read_bytes(code_sub_slice(5..)?, native_ptr(len))?.to_vec()))
			}
			15 => Ok(Self::Swap(read_u8(code_sub_slice(1..)?)?)),
			16 => Ok(Self::Write8(read_u8(code_sub_slice(1..)?)?)),
			17 => Ok(Self::Write16(read_u8(code_sub_slice(1..)?)?)),
			18 => Ok(Self::Write32(read_u8(code_sub_slice(1..)?)?)),
			19 => Ok(Self::ReadStackPointer),
			20 => Ok(Self::WriteStackPointer),
			21 => Ok(Self::Jump(read_vm_ptr(code_sub_slice(1..)?)?)),
			22 => Ok(Self::Call(read_vm_ptr(code_sub_slice(1..)?)?)),
			23 => Ok(Self::Return),
			24 => Ok(Self::Increment),
			25 => Ok(Self::Decrement),
			26 => Ok(Self::Add(read_u8(code_sub_slice(1..)?)?)),
			27 => Ok(Self::Sub(read_u8(code_sub_slice(1..)?)?)),
			28 => Ok(Self::Compare(read_u8(code_sub_slice(1..)?)?)),
			29 => Ok(Self::JumpEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			30 => Ok(Self::JumpNotEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			31 => Ok(Self::JumpGreater(read_vm_ptr(code_sub_slice(1..)?)?)),
			32 => Ok(Self::JumpLess(read_vm_ptr(code_sub_slice(1..)?)?)),
			33 => Ok(Self::JumpGreaterEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			34 => Ok(Self::JumpLessEqual(read_vm_ptr(code_sub_slice(1..)?)?)),
			35 => Ok(Self::JumpZero(read_vm_ptr(code_sub_slice(1..)?)?)),
			36 => Ok(Self::JumpNonzero(read_vm_ptr(code_sub_slice(1..)?)?)),
			37 => Ok(Self::Push),
			38 => Ok(Self::Pop),
			39 => Ok(Self::PushRegister(read_u8(code_sub_slice(1..)?)?)),
			40 => Ok(Self::PopRegister(read_u8(code_sub_slice(1..)?)?)),
			41 => Ok(Self::Mul(read_u8(code_sub_slice(1..)?)?)),
			42 => Ok(Self::Div(read_u8(code_sub_slice(1..)?)?)),
			43 => Ok(Self::IncrementRegister(read_u8(code_sub_slice(1..)?)?)),
			44 => Ok(Self::DecrementRegister(read_u8(code_sub_slice(1..)?)?)),
			45 => Ok(Self::SetRegister(
				read_u8(code_sub_slice(1..)?)?,
				read_vm_ptr(code_sub_slice(2..)?)?,
			)),
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
			Self::Set(value) => {
				bytes.push(8);
				bytes.extend_from_slice(&value.to_be_bytes());
			}
			Self::Deref8(reg) => {
				bytes.push(9);
				bytes.push(*reg);
			}
			Self::Deref16(reg) => {
				bytes.push(10);
				bytes.push(*reg);
			}
			Self::Deref32(reg) => {
				bytes.push(11);
				bytes.push(*reg);
			}
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
			Self::Swap(reg) => {
				bytes.push(15);
				bytes.push(*reg);
			}
			Self::Write8(reg) => {
				bytes.push(16);
				bytes.push(*reg);
			}
			Self::Write16(reg) => {
				bytes.push(17);
				bytes.push(*reg);
			}
			Self::Write32(reg) => {
				bytes.push(18);
				bytes.push(*reg);
			}
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
			Self::Add(reg) => {
				bytes.push(26);
				bytes.push(*reg);
			}
			Self::Sub(reg) => {
				bytes.push(27);
				bytes.push(*reg);
			}
			Self::Compare(reg) => {
				bytes.push(28);
				bytes.push(*reg);
			}
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
			Self::Push => bytes.push(37),
			Self::Pop => bytes.push(38),
			Self::PushRegister(reg) => {
				bytes.push(39);
				bytes.push(*reg);
			}
			Self::PopRegister(reg) => {
				bytes.push(40);
				bytes.push(*reg);
			}
			Self::Mul(reg) => {
				bytes.push(41);
				bytes.push(*reg);
			}
			Self::Div(reg) => {
				bytes.push(42);
				bytes.push(*reg);
			}
			Self::IncrementRegister(reg) => {
				bytes.push(43);
				bytes.push(*reg);
			}
			Self::DecrementRegister(reg) => {
				bytes.push(44);
				bytes.push(*reg);
			}
			Self::SetRegister(reg, value) => {
				bytes.push(45);
				bytes.push(*reg);
				bytes.extend_from_slice(&value.to_be_bytes());
			}
		}
		bytes
	}
}
