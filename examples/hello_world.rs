use my_vm::{Instruction, Machine, Program};

fn hello_world_program() -> anyhow::Result<Program> {
	let mut program = Program::new();
	// No-op.
	program.add_nop();
	// Add data segment to hold our string.
	let s = program.add_data(c"Hello world!".to_bytes_with_nul());
	// Load the data segment into machine memory at 10.
	program.add_copy_data(s, 10)?;
	// Set the main register to 10 to point to the address we wrote the string to.
	program.add_instruction(Instruction::Set(10));
	// Call syscall 0 (println).
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
