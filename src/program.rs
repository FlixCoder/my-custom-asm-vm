use std::{collections::HashMap, ffi::CString, mem::size_of, str::FromStr};

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
	pub fn add_data(&mut self, data: impl Into<Vec<u8>>) -> usize {
		let data = data.into();
		self.add_instruction(Instruction::Data(vm_ptr(data.len()), data))
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

	/// Add a dummy copy data instruction that needs to be adjusted later.
	/// Return the index of this instruction to be used by jumps or calls.
	pub fn add_dummy_copy_data(&mut self) -> usize {
		self.add_instruction(Instruction::CopyCodeMemory(VmPtr::MAX, 0))
	}

	/// Replace dummy copy data with real copy data instruction.
	pub fn replace_dummy_copy_data(
		&mut self,
		index: usize,
		data_index: usize,
	) -> anyhow::Result<()> {
		let (addr, instruction) = self.resolve(data_index).context("Invalid data index")?;
		let Instruction::Data(size, _data) = instruction else {
			return Err(anyhow::format_err!("Data index doesn't point to data"));
		};
		let source = addr + 1 + vm_ptr(size_of::<VmPtr>());
		let size = *size;
		let instruction = self.instructions.get_mut(index).context("Invalid instruction index")?;
		match instruction {
			Instruction::CopyCodeMemory(_, _) => {
				*instruction = Instruction::CopyCodeMemory(source, size);
			}
			_ => return Err(anyhow::format_err!("Instruction is not a dummy copy data")),
		}
		Ok(())
	}

	/// Add an instruction to the program that jumps to the indexed instruction.
	/// Return the index of this instruction to be used by jumps or calls.
	pub fn add_jump(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::Jump(addr));
		Ok(index)
	}

	/// Add dummy jump instruction to the program, that can and should later be
	/// altered to the correct jump address. Return the index of this
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

	/// Add dummy call instruction to the program, that can and should later be
	/// altered to the correct call address. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_call(&mut self) -> usize {
		self.add_instruction(Instruction::Call(VmPtr::MAX))
	}

	/// Add an instruction to the program that returns from a call. Return the
	/// index of this instruction to be used by jumps or calls.
	pub fn add_return(&mut self) -> usize {
		self.add_instruction(Instruction::Return)
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was equal. Return the index of this instruction
	/// to be used by jumps or calls.
	pub fn add_jump_equal(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpEqual(addr));
		Ok(index)
	}

	/// Add dummy jump equal instruction to the program, that can and should
	/// later be altered to the correct jump address. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_jump_equal(&mut self) -> usize {
		self.add_instruction(Instruction::JumpEqual(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was not equal. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_jump_not_equal(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpNotEqual(addr));
		Ok(index)
	}

	/// Add dummy jump not equal instruction to the program, that can and should
	/// later be altered to the correct jump address. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_jump_not_equal(&mut self) -> usize {
		self.add_instruction(Instruction::JumpNotEqual(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was greater. Return the index of this instruction
	/// to be used by jumps or calls.
	pub fn add_jump_greater(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpGreater(addr));
		Ok(index)
	}

	/// Add dummy jump greater instruction to the program, that can and should
	/// later be altered to the correct jump address. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_jump_greater(&mut self) -> usize {
		self.add_instruction(Instruction::JumpGreater(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was less. Return the index of this instruction
	/// to be used by jumps or calls.
	pub fn add_jump_less(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpLess(addr));
		Ok(index)
	}

	/// Add dummy jump less instruction to the program, that can and should
	/// later be altered to the correct jump address. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_dummy_jump_less(&mut self) -> usize {
		self.add_instruction(Instruction::JumpLess(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was greater or equal. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_jump_greater_equal(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpGreaterEqual(addr));
		Ok(index)
	}

	/// Add dummy jump greater equal instruction to the program, that can and
	/// should later be altered to the correct jump address. Return the index
	/// of this instruction to be used by jumps or calls.
	pub fn add_dummy_jump_greater_equal(&mut self) -> usize {
		self.add_instruction(Instruction::JumpGreaterEqual(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last comparison was less or equal. Return the index of this
	/// instruction to be used by jumps or calls.
	pub fn add_jump_less_equal(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpLessEqual(addr));
		Ok(index)
	}

	/// Add dummy jump less equal instruction to the program, that can and
	/// should later be altered to the correct jump address. Return the index
	/// of this instruction to be used by jumps or calls.
	pub fn add_dummy_jump_less_equal(&mut self) -> usize {
		self.add_instruction(Instruction::JumpLessEqual(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last increment/decrement resulted in zero. Return the index of
	/// this instruction to be used by jumps or calls.
	pub fn add_jump_zero(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpZero(addr));
		Ok(index)
	}

	/// Add dummy jump zero instruction to the program, that can and
	/// should later be altered to the correct jump address. Return the index
	/// of this instruction to be used by jumps or calls.
	pub fn add_dummy_jump_zero(&mut self) -> usize {
		self.add_instruction(Instruction::JumpZero(VmPtr::MAX))
	}

	/// Add an instruction to the program that jumps to the indexed instruction
	/// if the last increment/decrement resulted in nonzero. Return the index of
	/// this instruction to be used by jumps or calls.
	pub fn add_jump_nonzero(&mut self, index: usize) -> anyhow::Result<usize> {
		let (addr, _) = self.resolve(index).context("Invalid instruction index")?;
		let index = self.add_instruction(Instruction::JumpNonzero(addr));
		Ok(index)
	}

	/// Add dummy jump nonzero instruction to the program, that can and
	/// should later be altered to the correct jump address. Return the index
	/// of this instruction to be used by jumps or calls.
	pub fn add_dummy_jump_nonzero(&mut self) -> usize {
		self.add_instruction(Instruction::JumpNonzero(VmPtr::MAX))
	}

	/// Replace a dummy jump/call address with a real address. This is useful
	/// when the code that we want to jump to does not exist yet in the
	/// program.
	pub fn replace_dummy_address(&mut self, index: usize, jump_index: usize) -> anyhow::Result<()> {
		let (addr, _) = self.resolve(jump_index).context("Invalid jump index")?;
		let instruction = self.instructions.get_mut(index).context("Invalid instruction index")?;
		match instruction {
			Instruction::Call(jump)
			| Instruction::Jump(jump)
			| Instruction::JumpEqual(jump)
			| Instruction::JumpNotEqual(jump)
			| Instruction::JumpLess(jump)
			| Instruction::JumpGreater(jump)
			| Instruction::JumpGreaterEqual(jump)
			| Instruction::JumpLessEqual(jump)
			| Instruction::JumpZero(jump)
			| Instruction::JumpNonzero(jump)
				if *jump == VmPtr::MAX =>
			{
				*jump = addr
			}
			_ => return Err(anyhow::format_err!("Instruction is not a dummy jump or call")),
		}
		Ok(())
	}
}

impl FromStr for Program {
	type Err = anyhow::Error;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let mut program = Program::new();
		let mut next_index: usize = 0;
		let mut label_index = HashMap::new();
		let mut dummy_jumps = Vec::new();
		let mut dummy_copy_data = Vec::new();

		// Parse lines into instructions, making dummies at references to labels.
		for line in input.lines().map(str::trim).filter(|s| !s.is_empty()) {
			let parts = line.split_whitespace().collect::<Vec<_>>();
			match parts[0].to_lowercase().as_str() {
				// Comments.
				"#" | "//" => continue,
				// Label <name>
				"label" if parts.len() == 2 => {
					label_index.insert(parts[1], next_index);
				}
				// Nop
				"nop" if parts.len() == 1 => {
					program.add_nop();
					next_index += 1;
				}
				// Halt
				"halt" if parts.len() == 1 => {
					program.add_instruction(Instruction::Halt);
					next_index += 1;
				}
				// Load8 <ptr>
				"load8" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Load8(ptr));
					next_index += 1;
				}
				// Load16 <ptr>
				"store8" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Store8(ptr));
					next_index += 1;
				}
				// Load16 <ptr>
				"load16" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Load16(ptr));
					next_index += 1;
				}
				// Store16 <ptr>
				"store16" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Store16(ptr));
					next_index += 1;
				}
				// Load32 <ptr>
				"load32" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Load32(ptr));
					next_index += 1;
				}
				// Store32 <ptr>
				"store32" if parts.len() == 2 => {
					let ptr = parts[1].parse()?;
					program.add_instruction(Instruction::Store32(ptr));
					next_index += 1;
				}
				// Set <value>
				"set" if parts.len() == 2 => {
					let value = parts[1].parse()?;
					program.add_instruction(Instruction::Set(value));
					next_index += 1;
				}
				// Deref8 <register>
				"deref8" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Deref8(register));
					next_index += 1;
				}
				// Deref16 <register>
				"deref16" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Deref16(register));
					next_index += 1;
				}
				// Deref32 <register>
				"deref32" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Deref32(register));
					next_index += 1;
				}
				// Syscall <id>
				"syscall" if parts.len() == 2 => {
					let id = parts[1].parse()?;
					program.add_syscall(id);
					next_index += 1;
				}
				// CopyCodeMemory <target_data_label>
				"copycodememory" if parts.len() == 2 => {
					let index = program.add_dummy_copy_data();
					dummy_copy_data.push((index, parts[1]));
					next_index += 1;
				}
				// DataString <str>
				"datastring" => {
					let cstr = CString::new(line.split_at(10).1.trim())?;
					program.add_data(cstr.into_bytes_with_nul());
					next_index += 1;
				}
				// Swap <register>
				"swap" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Swap(register));
					next_index += 1;
				}
				// Write8 <register>
				"write8" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Write8(register));
					next_index += 1;
				}
				// Write16 <register>
				"write16" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Write16(register));
					next_index += 1;
				}
				// Write32 <register>
				"write32" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Write32(register));
					next_index += 1;
				}
				// ReadStackPointer
				"readstackpointer" if parts.len() == 1 => {
					program.add_instruction(Instruction::ReadStackPointer);
					next_index += 1;
				}
				// WriteStackPointer
				"writestackpointer" if parts.len() == 1 => {
					program.add_instruction(Instruction::WriteStackPointer);
					next_index += 1;
				}
				// Jump <label>
				"jump" if parts.len() == 2 => {
					let index = program.add_dummy_jump();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// Call <label>
				"call" if parts.len() == 2 => {
					let index = program.add_dummy_call();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// Return
				"return" if parts.len() == 1 => {
					program.add_instruction(Instruction::Return);
					next_index += 1;
				}
				// Increment
				"increment" if parts.len() == 1 => {
					program.add_instruction(Instruction::Increment);
					next_index += 1;
				}
				// Decrement
				"decrement" if parts.len() == 1 => {
					program.add_instruction(Instruction::Decrement);
					next_index += 1;
				}
				// Add <register>
				"add" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Add(register));
					next_index += 1;
				}
				// Sub <register>
				"sub" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Sub(register));
					next_index += 1;
				}
				// Compare <register>
				"compare" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Compare(register));
					next_index += 1;
				}
				// JumpEqual <label>
				"jumpequal" if parts.len() == 2 => {
					let index = program.add_dummy_jump_equal();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpNotEqual <label>
				"jumpnotequal" if parts.len() == 2 => {
					let index = program.add_dummy_jump_not_equal();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpGreater <label>
				"jumpgreater" if parts.len() == 2 => {
					let index = program.add_dummy_jump_greater();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpLess <label>
				"jumpless" if parts.len() == 2 => {
					let index = program.add_dummy_jump_less();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpGreaterEqual <label>
				"jumpgreaterequal" if parts.len() == 2 => {
					let index = program.add_dummy_jump_greater_equal();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpLessEqual <label>
				"jumplessequal" if parts.len() == 2 => {
					let index = program.add_dummy_jump_less_equal();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpZero <label>
				"jumpzero" if parts.len() == 2 => {
					let index = program.add_dummy_jump_zero();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// JumpNonzero <label>
				"jumpnonzero" if parts.len() == 2 => {
					let index = program.add_dummy_jump_nonzero();
					dummy_jumps.push((index, parts[1]));
					next_index += 1;
				}
				// Push
				"push" if parts.len() == 1 => {
					program.add_instruction(Instruction::Push);
					next_index += 1;
				}
				// Pop
				"pop" if parts.len() == 1 => {
					program.add_instruction(Instruction::Pop);
					next_index += 1;
				}
				// PushRegister <register>
				"pushregister" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::PushRegister(register));
					next_index += 1;
				}
				// PopRegister <register>
				"popregister" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::PopRegister(register));
					next_index += 1;
				}
				// Mul <register>
				"mul" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Mul(register));
					next_index += 1;
				}
				// Div <register>
				"div" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::Div(register));
					next_index += 1;
				}
				// IncrementRegister <register>
				"incrementregister" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::IncrementRegister(register));
					next_index += 1;
				}
				// DecrementRegister <register>
				"decrementregister" if parts.len() == 2 => {
					let register = parts[1].parse()?;
					program.add_instruction(Instruction::DecrementRegister(register));
					next_index += 1;
				}
				// SetRegister <register> <value>
				"setregister" if parts.len() == 3 => {
					let register = parts[1].parse()?;
					let value = parts[2].parse()?;
					program.add_instruction(Instruction::SetRegister(register, value));
					next_index += 1;
				}
				// Unknown command.
				cmd => {
					return Err(anyhow::format_err!(
						"Unknown command or wrong number of arguments: {cmd}"
					))
				}
			}
		}

		// Resolve dummies to their labels.
		for (index, label) in dummy_jumps {
			let target =
				*label_index.get(&label).with_context(|| format!("Unresolved label: {label}"))?;
			program.replace_dummy_address(index, target)?;
		}
		for (index, label) in dummy_copy_data {
			let target =
				*label_index.get(&label).with_context(|| format!("Unresolved label: {label}"))?;
			program.replace_dummy_copy_data(index, target)?;
		}

		Ok(program)
	}
}
