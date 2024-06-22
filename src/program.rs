use std::mem::size_of;

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

	/// Add an instruction to the program.
	pub fn add_instruction(&mut self, instruction: Instruction) {
		self.instructions.push(instruction);
	}

	/// Add NOP instruction to the program.
	pub fn add_nop(&mut self) {
		self.instructions.push(Instruction::Nop);
	}

	/// Add a halt instruction to the program.
	pub fn add_halt(&mut self) {
		self.instructions.push(Instruction::Halt);
	}

	/// Add a syscall instruction to the program.
	pub fn add_syscall(&mut self, index: u8) {
		self.instructions.push(Instruction::Syscall(index));
	}

	/// Add a data segment to the program. Returns the index of this instruction
	/// to be used in [`make_copy_data`].
	pub fn add_data(&mut self, data: &[u8]) -> usize {
		self.instructions.push(Instruction::Data(vm_ptr(data.len()), data.into()));
		self.instructions.len() - 1
	}

	/// Add an instruction to the program that copies the data from the indexed
	/// data segment to the target address in machine memory.
	pub fn add_copy_data(
		&mut self,
		for_data_index: usize,
		target_mem_addr: u32,
	) -> anyhow::Result<()> {
		let source: usize = self
			.instructions
			.get(..for_data_index)
			.iter()
			.copied()
			.flatten()
			.map(|i| i.size())
			.sum();
		let Some(Instruction::Data(size, _data)) = self.instructions.get(for_data_index) else {
			return Err(anyhow::format_err!("Invalid data index {for_data_index}"));
		};
		let source = vm_ptr(source + 1 + size_of::<VmPtr>());
		self.instructions.push(Instruction::CopyCodeMemory(source, target_mem_addr, *size));
		Ok(())
	}
}
