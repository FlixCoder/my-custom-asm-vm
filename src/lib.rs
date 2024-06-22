mod instruction;
mod program;
mod util;

use std::mem::size_of;

use anyhow::Context;
use util::{
	native_ptr, read_cstr, read_u16, read_u32, read_u8, read_vm_ptr, vm_ptr, write_u16, write_u32,
	write_u8, write_vm_ptr,
};

pub use crate::{instruction::Instruction, program::Program};

/// VM pointer size.
pub type VmPtr = u32;

/// Virtual machine for my custom binary assembler language.
#[derive(Debug, PartialEq, Clone)]
pub struct Machine {
	program: Box<[u8]>,
	memory: Box<[u8]>,
	instruction_pointer: VmPtr,
	stack_pointer: VmPtr,
	main_register: VmPtr,
	register_a: VmPtr,
}

impl Machine {
	/// Create a new virtual machine with the given program and memory size.
	/// Stack pointer is initally at the end of the memory.
	pub fn new(program: impl Into<Box<[u8]>>, memory_size: VmPtr) -> Self {
		Self {
			program: program.into(),
			memory: vec![0; native_ptr(memory_size)].into(),
			instruction_pointer: 0,
			stack_pointer: memory_size,
			main_register: 0,
			register_a: 0,
		}
	}

	/// Get byte slice at the given memory pointer.
	fn memory(&self, ptr: VmPtr) -> anyhow::Result<&[u8]> {
		self.memory
			.get(native_ptr(ptr)..)
			.with_context(|| format!("Out of memory access occured at {ptr}"))
	}

	/// Get mutable byte slice at the given memory pointer.
	fn memory_mut(&mut self, ptr: VmPtr) -> anyhow::Result<&mut [u8]> {
		self.memory
			.get_mut(native_ptr(ptr)..)
			.with_context(|| format!("Out of memory access occured at {ptr}"))
	}

	/// Make a syscall at the current state.
	///
	/// Available syscalls:
	/// - 0: Print the string referenced by the main register.
	fn syscall(&mut self, index: u8) -> anyhow::Result<()> {
		match index {
			0 => {
				let mem = self.memory(self.main_register)?;
				let cstr = read_cstr(mem)?;
				let s = cstr.to_str().with_context(|| {
					format!("Accessed invalid string at {}", self.main_register)
				})?;
				println!("{s}");
			}
			_ => return Err(anyhow::format_err!("Unknown syscall {index}")),
		}
		Ok(())
	}

	/// Run a step of the virtual machine. Return whether the execution should
	/// continue.
	#[allow(clippy::unnecessary_cast, clippy::useless_conversion)] // For future compatibility, when changing VmPtr.
	pub fn step(&mut self) -> anyhow::Result<bool> {
		let code = self
			.program
			.get(native_ptr(self.instruction_pointer)..)
			.context("Instruction pointer is outside of program code")?;
		let instruction = Instruction::parse(code).context("Failed parsing instruction")?;
		self.instruction_pointer += vm_ptr(instruction.size());
		match instruction {
			Instruction::Nop | Instruction::Data(_, _) => {}
			Instruction::Halt => return Ok(false),
			Instruction::Load8(ptr) => {
				let mem = self.memory(ptr)?;
				self.main_register = read_u8(mem)?.into();
			}
			Instruction::Store8(ptr) => {
				let value = self.main_register as u8;
				let mem = self.memory_mut(ptr)?;
				write_u8(mem, value)?;
			}
			Instruction::Load16(ptr) => {
				let mem = self.memory(ptr)?;
				self.main_register = read_u16(mem)?.into();
			}
			Instruction::Store16(ptr) => {
				let value = self.main_register as u16;
				let mem = self.memory_mut(ptr)?;
				write_u16(mem, value)?;
			}
			Instruction::Load32(ptr) => {
				let mem = self.memory(ptr)?;
				self.main_register = read_u32(mem)?.into();
			}
			Instruction::Store32(ptr) => {
				let value = self.main_register as u32;
				let mem = self.memory_mut(ptr)?;
				write_u32(mem, value)?;
			}
			Instruction::Set(value) => self.main_register = value,
			Instruction::Deref8 => {
				let mem = self.memory(self.main_register)?;
				self.main_register = read_u8(mem)?.into();
			}
			Instruction::Deref16 => {
				let mem = self.memory(self.main_register)?;
				self.main_register = read_u16(mem)?.into();
			}
			Instruction::Deref32 => {
				let mem = self.memory(self.main_register)?;
				self.main_register = read_u32(mem)?.into();
			}
			Instruction::Syscall(index) => self.syscall(index)?,
			Instruction::CopyCodeMemory(source, size) => {
				let source = native_ptr(source);
				let target = native_ptr(self.main_register);
				let size = native_ptr(size);
				let source = self.program.get(source..(source + size)).with_context(|| {
					format!("Out of memory access occurred at program memory {source}")
				})?;
				let target = self
					.memory
					.get_mut(target..(target + size))
					.with_context(|| format!("Out of memory access occurred at {target}"))?;
				target.copy_from_slice(source);
			}
			Instruction::SwapRegisterA => {
				std::mem::swap(&mut self.main_register, &mut self.register_a)
			}
			Instruction::Write8 => {
				let value = self.register_a as u8;
				let mem = self.memory_mut(self.main_register)?;
				write_u8(mem, value)?;
			}
			Instruction::Write16 => {
				let value = self.register_a as u16;
				let mem = self.memory_mut(self.main_register)?;
				write_u16(mem, value)?;
			}
			Instruction::Write32 => {
				let value = self.register_a as u32;
				let mem = self.memory_mut(self.main_register)?;
				write_u32(mem, value)?;
			}
			Instruction::ReadStackPointer => self.main_register = self.stack_pointer,
			Instruction::WriteStackPointer => self.stack_pointer = self.main_register,
			Instruction::Jump(addr) => self.instruction_pointer = addr,
			Instruction::Call(addr) => {
				self.stack_pointer = self
					.stack_pointer
					.checked_sub(vm_ptr(size_of::<VmPtr>()))
					.context("Stack overflow")?;
				let ip = self.instruction_pointer;
				let mem = self.memory_mut(self.stack_pointer)?;
				write_vm_ptr(mem, ip)?;
				self.instruction_pointer = addr;
			}
			Instruction::Return => {
				let mem = self.memory(self.stack_pointer)?;
				self.instruction_pointer = read_vm_ptr(mem)?;
				self.stack_pointer = self
					.stack_pointer
					.checked_add(vm_ptr(size_of::<VmPtr>()))
					.context("Stack underflow")?;
			}
		}
		Ok(true)
	}

	/// Run the virtual machine until it halts (or errors).
	pub fn run(&mut self) -> anyhow::Result<()> {
		while self.step()? {}
		Ok(())
	}
}
