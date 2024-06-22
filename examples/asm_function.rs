use my_vm::{Machine, Program};

const PROGRAM: &str = r#"
// Jump straight to the main function.
jump main

// Data segment to hold our string.
label data
dataString Hello world!

// Function to print the string.
label function
// Set the main register to 0 to point to the address we want to write the string to.
set 0
// Load the data segment into machine memory at the address in the main register.
copyCodeMemory data
// Call syscall 0 (println). Reads the string from the address in the main register.
syscall 0
// Return from the function.
return

// Main function.
label main
// Call the function 5 times.
call function
call function
call function
call function
call function

// Halt the machine.
halt
"#;

fn main() -> anyhow::Result<()> {
	let program: Program = PROGRAM.parse()?;
	let executable = program.compile();

	let mut machine = Machine::new(executable, 1024);
	machine.run()?;
	Ok(())
}

#[test]
fn test() {
	main().unwrap();
}
