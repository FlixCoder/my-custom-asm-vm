use my_vm::{Instruction, Machine, Program};

fn hello_world_program() -> anyhow::Result<Program> {
	let mut program = Program::new();
	// Add data segment to hold our string.
	let s = program.add_data(c"Hello world!".to_bytes_with_nul());
	// Set the main register to 10 to point to the address we want to write the
	// string to.
	program.add_instruction(Instruction::Set(10));
	// Load the data segment into machine memory at the address in the main
	// register.
	program.add_copy_data(s)?;
	// Call syscall 0 (println). Reads the string from the address in the main
	// register.
	program.add_syscall(0);
	// Halt the machine.
	program.add_halt();
	Ok(program)
}

fn main() -> anyhow::Result<()> {
	let program = hello_world_program()?;
	let executable = program.compile();

	let mut machine = Machine::new(executable, 1024);
	machine.run()?;
	Ok(())
}
