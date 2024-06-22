use my_vm::{Machine, Program};

const PROGRAM: &str = r#"
# Add data segment to hold our string.
label str
dataString Hello world!

# Set the main register to 10 to point to the address we want to write the string to.
set 10
# Load the data segment into machine memory at the address in the main register.
copyCodeMemory str
# Call syscall 0 (println). Reads the string from the address in the main register.
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
