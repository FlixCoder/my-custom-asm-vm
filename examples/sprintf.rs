use my_vm::{Machine, Program};

const PROGRAM: &str = r#"
# Start from main.
jump main

# Function: itoa
# Converts the number in the main register to a string at the memory address given in side register 0.
# Returns the number of written characters in the main register.
# Uses side registers 0-3.
label itoa
# Set up.
swap 2
set 48
swap 3
set 0
swap 2
# Loop: Divide the number by 10 and write the remainder to the string.
# r0: Memory address of the string.
# r1: Divisor/remainder.
# r2: Counter of characters.
# r3: '0'=48 to make numbers to characters.
label itoa_loop_1
swap 1
set 10
swap 1
div 1
swap 1
add 3
swap 1
swap 0
write8 1
increment
swap 0
swap 2
increment
swap 2
increment
decrement
jumpNonzero itoa_loop_1
# Write 0 to the end of the string (r0 is currently 0).
swap 0
write8 0
swap 0
# We are done if there is only 1 character.
set 1
compare 2
jumpLess itoa_reverse
swap 2
return
# Reverse the numbers in the string.
# r0: Memory address of the string from the end.
# r1: Memory address of the string from the beginning.
# r2: Counter of characters.
# r3: Intermediate character value.
label itoa_reverse
set 1
swap 3
swap 0
push
decrement
swap 0
pop
sub 2
swap 1
label itoa_loop_2
deref8 0
swap 3
deref8 1
swap 0
write8 0
decrement
swap 0
swap 1
write8 3
increment
compare 0
swap 1
jumpLess itoa_loop_2
swap 2
return

# Function: copy_str
# Copies a string from the memory address given in the main register to the memory address given in side register 0.
# Returns the number of written characters in the main register.
# Uses side registers 0-3:
# r0: Memory address of the target string.
# r1: Memory address of the source string.
# r2: Character counter.
# r3: 0 for comparison.
label copy_str
swap 1
set 0
swap 2
set 0
swap 3
jump copy_str_first_iteration
label copy_str_loop
swap 2
increment
swap 2
label copy_str_first_iteration
deref8 1
swap 1
increment
swap 1
swap 0
write8 0
increment
swap 0
compare 3
jumpNotEqual copy_str_loop
swap 2
return

# Function: sprintf
# Prints a formatted string. Main register must be memory address of target string.
# Side register 0 must be format string, e.g. "Hello %s: %d!" will read 2 arguments: a string and a number.
# Side register 1 references a list of arguments (either a number or a pointer to string).
# Uses side registers 0-2:
# r0: Pointer to the format string.
# r1: Pointer to the list of arguments.
# r2: Pointer to the target string.
# r3: 0, '%', '%s' or '%d' for comparison or just something intermediate.
label sprintf
swap 2
# Loop: copy characters from format string to target string, but insert arguments when it should.
label sprintf_loop
set 37
swap 3
deref8 0
swap 0
increment
swap 0
compare 3
jumpNotEqual sprintf_copy
# %s or %d.
set 115
swap 3
deref8 0
swap 0
increment
swap 0
compare 3
jumpNotEqual sprintf_%d
# %s.
pushRegister 0
pushRegister 1
pushRegister 2
swap 2
swap 0
deref32 1
call copy_str
popRegister 2
popRegister 1
popRegister 0
add 2
swap 2
set 4
swap 3
swap 1
add 3
swap 1
jump sprintf_loop
# %d.
label sprintf_%d
pushRegister 0
pushRegister 1
pushRegister 2
swap 2
swap 0
deref32 1
call itoa
popRegister 2
popRegister 1
popRegister 0
add 2
swap 2
set 4
swap 3
swap 1
add 3
swap 1
jump sprintf_loop
label sprintf_copy
swap 2
write8 2
increment
swap 2
swap 3
set 0
swap 3
compare 3
jumpNotEqual sprintf_loop
return

label format_str
dataString Hello %s: %d!
label inner_str
dataString world

# Main.
label main
# Put arguments to memory[0..8], inner string to memory[8..50], format string to memory[50..100].
set 8
store32 0
copyCodeMemory inner_str
set 123456789
store32 4
set 50
copyCodeMemory format_str
# Put format_str pointer to side register 0, arguments pointer to side register 1 and target pointer to main register.
set 50
swap 0
set 0
swap 1
set 100
call sprintf
set 100
syscall 0
halt
"#;

fn main() -> anyhow::Result<()> {
	let program: Program = PROGRAM.parse()?;
	let executable = program.compile();

	// Machine with 4 side registers and 1024 bytes memory.
	let mut machine = Machine::<4>::new(executable, 1024);
	machine.run()?;
	Ok(())
}

#[test]
fn test() {
	main().unwrap();
}
