mod instruction;
mod program;
mod util;

use anyhow::Context;
use util::{
	native_ptr, read_cstr, read_u16, read_u32, read_u8, vm_ptr, write_u16, write_u32, write_u8,
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
	main_register: VmPtr,
}

impl Machine {
	/// Create a new virtual machine with the given program and memory size.
	pub fn new(program: impl Into<Box<[u8]>>, memory_size: VmPtr) -> Self {
		Self {
			program: program.into(),
			memory: vec![0; native_ptr(memory_size)].into(),
			instruction_pointer: 0,
			main_register: 0,
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
			Instruction::CopyCodeMemory(source, target, size) => {
				let source = native_ptr(source);
				let target = native_ptr(target);
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
		}
		Ok(true)
	}

	/// Run the virtual machine until it halts (or errors).
	pub fn run(&mut self) -> anyhow::Result<()> {
		while self.step()? {}
		Ok(())
	}
}
