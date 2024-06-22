use anyhow::Context;
use my_vm::{Machine, Program};

fn main() -> anyhow::Result<()> {
	let asm = std::fs::read_to_string("./program.asm").context("Cannot read ./program.asm file")?;
	let program = asm.parse::<Program>()?;
	let executable = program.compile();

	let mut machine = Machine::<8>::new(executable, 4096);
	machine.run()?;
	Ok(())
}
