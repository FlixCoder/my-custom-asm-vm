# My custom assembly VM

Small experimental project with a virtual machine for a custom instruction set.

## Fibonacci example

```asm
# Start from main.
jump main

# Function: Fibonacci (recursive).
# Computes the fibonacci number of the value in the main register.
# Returns the result in the main register.
# Modifies the side registers 0 and 1.
label fibonacci
# If n < 2, return 1.
swap 0
set 2
swap 0
compare 0
jumpGreater fibonacci_continue
set 1
return
label fibonacci_continue
# Otherwise Add fibonacci(n-1) + fibonacci(n-2).
decrement
push
call fibonacci
swap 1
pop
pushRegister 1
decrement
call fibonacci
popRegister 1
add 1
return

# Function: Print number on a line.
# Does not modify the register, but sets memory[0]=0.
label print_number
syscall 1
push
set 0
store8 0
syscall 0
pop
return

# Main.
label main
set 15
call fibonacci
call print_number
halt
```
