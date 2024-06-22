use my_vm::{Instruction, Machine, Program};

fn function_program() -> anyhow::Result<Program> {
	let mut program = Program::new();
	let start = program.add_dummy_jump();
	// Add data segment to hold our string.
	let data = program.add_data(c"Hello world!".to_bytes_with_nul());
	// Set the main register to 0 to point to the address we want to write the
	// string to.
	let function = program.add_instruction(Instruction::Set(0));
	// Load the data segment into machine memory at the address in the main
	// register.
	program.add_copy_data(data)?;
	// Call syscall 0 (println). Reads the string from the address in the main
	// register.
	program.add_syscall(0);
	// Return from the function.
	program.add_return();
	// Actual main start.
	let main = program.add_nop();
	// Jump straight to main from start.
	program.replace_dummy_address(start, main)?;
	// Call the function 5 times.
	program.add_call(function)?;
	program.add_call(function)?;
	program.add_call(function)?;
	program.add_call(function)?;
	program.add_call(function)?;
	// Halt the machine.
	program.add_halt();
	Ok(program)
}

fn main() -> anyhow::Result<()> {
	let program = function_program()?;
	let executable = program.compile();

	let mut machine = Machine::new(executable, 1024);
	machine.run()?;
	Ok(())
}

#[test]
fn test() {
	main().unwrap();
}
