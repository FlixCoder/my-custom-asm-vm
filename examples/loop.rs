use my_vm::{Instruction, Machine, Program};

fn loop_program() -> anyhow::Result<Program> {
	let mut program = Program::new();
	// Set main register to 5.
	program.add_instruction(Instruction::Set(5));
	// Start the for loop: Print the current value.
	let for_loop = program.add_syscall(1);
	// Decrement value, setting the zero flag.
	program.add_instruction(Instruction::Decrement);
	// Jump to the start of the loop if the value is not zero.
	program.add_jump_nonzero(for_loop)?;
	// Print empty string for newline.
	program.add_instruction(Instruction::Set(0));
	program.add_instruction(Instruction::Store8(0));
	program.add_syscall(0);
	// Halt the machine.
	program.add_halt();
	Ok(program)
}

fn main() -> anyhow::Result<()> {
	let program = loop_program()?;
	let executable = program.compile();

	let mut machine = Machine::<0>::new(executable, 1024);
	machine.run()?;
	Ok(())
}

#[test]
fn test() {
	main().unwrap();
}
