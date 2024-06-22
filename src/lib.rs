mod instruction;
mod program;
mod util;

use std::{cmp::Ordering, mem::size_of};

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
pub struct Machine<const SIDE_REGS: usize = 4> {
	program: Box<[u8]>,
	memory: Box<[u8]>,
	instruction_pointer: VmPtr,
	stack_pointer: VmPtr,
	main_register: VmPtr,
	side_registers: [VmPtr; SIDE_REGS],
	flag_zero: bool,
	flag_comparison: Ordering,
}

impl<const SIDE_REGS: usize> Machine<SIDE_REGS> {
	/// Create a new virtual machine with the given program and memory size.
	/// Stack pointer is initally at the end of the memory.
	pub fn new(program: impl Into<Box<[u8]>>, memory_size: VmPtr) -> Self {
		Self {
			program: program.into(),
			memory: vec![0; native_ptr(memory_size)].into(),
			instruction_pointer: 0,
			stack_pointer: memory_size,
			main_register: 0,
			side_registers: [0; SIDE_REGS],
			flag_zero: true,
			flag_comparison: Ordering::Equal,
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

	/// Get side register value.
	fn side_register(&self, reg: u8) -> anyhow::Result<VmPtr> {
		let register: usize = reg.into();
		self.side_registers
			.get(register)
			.copied()
			.with_context(|| format!("Side register {reg} out of bounds"))
	}

	/// Get side register mut.
	fn side_register_mut(&mut self, reg: u8) -> anyhow::Result<&mut VmPtr> {
		let register: usize = reg.into();
		self.side_registers
			.get_mut(register)
			.with_context(|| format!("Side register {reg} out of bounds"))
	}

	/// Make a syscall at the current state.
	///
	/// Available syscalls:
	/// - 0: Print line with the string referenced by the main register.
	/// - 1: Print the number in the main register.
	/// - 2: Print the string referenced by the main registern.
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
			1 => {
				print!("{}", self.main_register);
			}
			2 => {
				let mem = self.memory(self.main_register)?;
				let cstr = read_cstr(mem)?;
				let s = cstr.to_str().with_context(|| {
					format!("Accessed invalid string at {}", self.main_register)
				})?;
				print!("{s}");
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
			Instruction::Deref8(reg) => {
				let ptr = self.side_register(reg)?;
				let mem = self.memory(ptr)?;
				self.main_register = read_u8(mem)?.into();
			}
			Instruction::Deref16(reg) => {
				let ptr = self.side_register(reg)?;
				let mem = self.memory(ptr)?;
				self.main_register = read_u16(mem)?.into();
			}
			Instruction::Deref32(reg) => {
				let ptr = self.side_register(reg)?;
				let mem = self.memory(ptr)?;
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
			Instruction::Swap(reg) => {
				let register: usize = reg.into();
				std::mem::swap(
					&mut self.main_register,
					self.side_registers
						.get_mut(register)
						.with_context(|| format!("Side register {reg} out of bounds"))?,
				)
			}
			Instruction::Write8(reg) => {
				let value = self.main_register as u8;
				let mem = self.memory_mut(self.side_register(reg)?)?;
				write_u8(mem, value)?;
			}
			Instruction::Write16(reg) => {
				let value = self.main_register as u16;
				let mem = self.memory_mut(self.side_register(reg)?)?;
				write_u16(mem, value)?;
			}
			Instruction::Write32(reg) => {
				let value = self.main_register as u32;
				let mem = self.memory_mut(self.side_register(reg)?)?;
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
			Instruction::Increment => {
				self.main_register = self.main_register.wrapping_add(1);
				self.flag_zero = self.main_register == 0;
			}
			Instruction::Decrement => {
				self.main_register = self.main_register.wrapping_sub(1);
				self.flag_zero = self.main_register == 0;
			}
			Instruction::Add(reg) => {
				self.main_register = self.main_register.wrapping_add(self.side_register(reg)?)
			}
			Instruction::Sub(reg) => {
				self.main_register = self.main_register.wrapping_sub(self.side_register(reg)?)
			}
			Instruction::Compare(reg) => {
				self.flag_comparison = self.main_register.cmp(&self.side_register(reg)?)
			}
			Instruction::JumpEqual(addr) => {
				if self.flag_comparison == Ordering::Equal {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpNotEqual(addr) => {
				if self.flag_comparison != Ordering::Equal {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpGreater(addr) => {
				if self.flag_comparison == Ordering::Greater {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpLess(addr) => {
				if self.flag_comparison == Ordering::Less {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpGreaterEqual(addr) => {
				if self.flag_comparison != Ordering::Less {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpLessEqual(addr) => {
				if self.flag_comparison != Ordering::Greater {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpZero(addr) => {
				if self.flag_zero {
					self.instruction_pointer = addr;
				}
			}
			Instruction::JumpNonzero(addr) => {
				if !self.flag_zero {
					self.instruction_pointer = addr;
				}
			}
			Instruction::Push => {
				self.stack_pointer = self
					.stack_pointer
					.checked_sub(vm_ptr(size_of::<VmPtr>()))
					.context("Stack overflow")?;
				let value = self.main_register;
				let mem = self.memory_mut(self.stack_pointer)?;
				write_vm_ptr(mem, value)?;
			}
			Instruction::Pop => {
				let mem = self.memory(self.stack_pointer)?;
				self.main_register = read_vm_ptr(mem)?;
				self.stack_pointer = self
					.stack_pointer
					.checked_add(vm_ptr(size_of::<VmPtr>()))
					.context("Stack underflow")?;
			}
			Instruction::PushRegister(reg) => {
				self.stack_pointer = self
					.stack_pointer
					.checked_sub(vm_ptr(size_of::<VmPtr>()))
					.context("Stack overflow")?;
				let value = self.side_register(reg)?;
				let mem = self.memory_mut(self.stack_pointer)?;
				write_vm_ptr(mem, value)?;
			}
			Instruction::PopRegister(reg) => {
				let mem = self.memory(self.stack_pointer)?;
				let value = read_vm_ptr(mem)?;
				let register = self.side_register_mut(reg)?;
				*register = value;
				self.stack_pointer = self
					.stack_pointer
					.checked_add(vm_ptr(size_of::<VmPtr>()))
					.context("Stack underflow")?;
			}
			Instruction::Mul(reg) => {
				self.main_register = self.main_register.wrapping_mul(self.side_register(reg)?)
			}
			Instruction::Div(reg) => {
				let value = self.main_register;
				let register = self.side_register_mut(reg)?;
				if *register == 0 {
					anyhow::bail!("Division by zero");
				}
				let divisor = *register;
				*register = value % divisor;
				self.main_register = value / divisor;
			}
			Instruction::IncrementRegister(reg) => {
				let register = self.side_register_mut(reg)?;
				*register = register.wrapping_add(1);
				self.flag_zero = *register == 0;
			}
			Instruction::DecrementRegister(reg) => {
				let register = self.side_register_mut(reg)?;
				*register = register.wrapping_sub(1);
				self.flag_zero = *register == 0;
			}
			Instruction::SetRegister(reg, value) => {
				let register = self.side_register_mut(reg)?;
				*register = value;
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
