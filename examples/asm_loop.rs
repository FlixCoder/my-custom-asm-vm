use my_vm::{Machine, Program};

const PROGRAM: &str = r#"
# Set main register to 5.
set 5

// Start the for loop.
label for_loop
// Print the current value.
syscall 1
// Decrement value, setting the zero flag.
decrement
// Jump to the start of the loop if the value is not zero.
jumpNonzero for_loop

# Print empty string for newline.
set 0
store8 0
syscall 0

# Halt the machine.
halt
"#;

fn main() -> anyhow::Result<()> {
	let program: Program = PROGRAM.parse()?;
	let executable = program.compile();

	let mut machine = Machine::<0>::new(executable, 1024);
	machine.run()?;
	Ok(())
}

#[test]
fn test() {
	main().unwrap();
}
