use std::mem::size_of;

use anyhow::Context;

use crate::{instruction::Instruction, util::vm_ptr, VmPtr};

/// A full programm. Just a helper to create programs, the VM uses actual byte
/// code.
#[derive(Debug, Clone, Default)]
pub struct Program {
	instructions: Vec<Instruction>,
}

impl Program {
	/// Create new empty program.
	pub fn new() -> Self {
		Self::default()
	}

	/// Compile the program to continuous bytes.
	pub fn compile(&self) -> Vec<u8> {
		self.instructions.iter().flat_map(|i| i.bytes()).collect()
	}

	/// Add an instruction to the program. Return the index of this instruction
	/// to be used by jumps or calls.
	pub fn add_instruction(&mut self, instruction: Instruction) -> usize {
		self.instructions.push(instruction);
		self.instructions.len() - 1
	}

	/// Add NOP instruction to the program. Return the index of this instruction
	/// to be used by jumps or calls.
	pub fn add_nop(&mut self) -> usize {
		self.add_instruction(Instruction::Nop)
	}

	/// Add a halt instruction to the program. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_halt(&mut self) -> usize {
		self.add_instruction(Instruction::Halt)
	}

	/// Add a syscall instruction to the program. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_syscall(&mut self, index: u8) -> usize {
		self.add_instruction(Instruction::Syscall(index))
	}

	/// Add a data segment to the program. Returns the index of this instruction
	/// to be used in [`make_copy_data`].
	pub fn add_data(&mut self, data: &[u8]) -> usize {
		self.add_instruction(Instruction::Data(vm_ptr(data.len()), data.into()))
	}

	/// Resolve the instruction index to a code memory address and its
	/// instruction.
	fn resolve(&self, index: usize) -> Option<(VmPtr, &Instruction)> {
		let addr = self.instructions.iter().take(index).map(|i| vm_ptr(i.size())).sum();
		let instruction = self.instructions.get(index)?;
		Some((addr, instruction))
	}

	/// Add an instruction to the program that copies the data from the indexed
	/// data segment to the target address in machine memory. Return the index
	/// of this instruction to be used by jumps or calls.
	pub fn add_copy_data(&mut self, for_data_index: usize) -> anyhow::Result<usize> {
		let (addr, instruction) = self.resolve(for_data_index).context("Invalid data index")?;
		let Instruction::Data(size, _data) = instruction else {
			return Err(anyhow::format_err!("Data index doesn't point to data"));
		};
		let source = addr + 1 + vm_ptr(size_of::<VmPtr>());
		let index = self.add_instruction(Instruction::CopyCodeMemory(source, *size));
		Ok(index)
	}

	/// Add an instruction to the program that jumps to the indexed instruction.
	/// Return the index of this instruction to be used by jumps or calls.
	pub fn add_jump(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::Jump(addr));
		Ok(index)
	}

	/// Add dummy jump instruction to the program. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_jump(&mut self) -> usize {
		self.add_instruction(Instruction::Jump(VmPtr::MAX))
	}

	/// Add an instruction to the program that call the indexed instruction.
	/// Return the index of this instruction to be used by jumps or calls.
	pub fn add_call(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::Call(addr));
		Ok(index)
	}

	/// Add dummy call instruction to the program. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_call(&mut self) -> usize {
		self.add_instruction(Instruction::Call(VmPtr::MAX))
	}

	/// Replace a jump address in an instruction in the program. This is useful
	/// when adding a dummy jump or call, because the code that we want to jump
	/// to does not exist yet.
	pub fn replace_jump_call_address(
		&mut self,
		index: usize,
		jump_index: usize,
	) -> anyhow::Result<()> {
		let (addr, _) = self.resolve(jump_index).context("Invalid jump index")?;
		let instruction = self.instructions.get_mut(index).context("Invalid instruction index")?;
		match instruction {
			Instruction::Jump(jump) => *jump = addr,
			Instruction::Call(jump) => *jump = addr,
			_ => return Err(anyhow::format_err!("Instruction is not a jump or call")),
		}
		Ok(())
	}

	/// Add an instruction to the program that returns from a call. Return the
	/// index of this instruction to be used by jumps or calls.
	pub fn add_return(&mut self) -> usize {
		self.add_instruction(Instruction::Return)
	}
}
