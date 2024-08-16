# lunalint

lunalint is a linter for Lua scripts. It is designed to be fast and to emit user-friendly messages.

## Prerequisites
- Cargo and Rust

## Build & Run 

```sh
cargo run --release -- <FILE>
```

## Example

```sh
$ cargo run --release -- examples/bad_code.lua 
[999] Error: Count down loop which never reaches end
   ╭─[bad_code.lua:1:5]
   │
 1 │ for i=10, 1 do
   │     ┬  
   │     ╰── This should be decreasing
───╯
[999] Error: The environment is set to nil
   ╭─[bad_code.lua:5:1]
   │
 5 │ _ENV = nil
   │ ──┬─  
   │   ╰─── Assignment occurs here
───╯
```
