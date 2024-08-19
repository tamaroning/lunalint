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
[999] Error: Count down loop which never reaches end (count-down-loop)
   ╭─[bad_code.lua:3:9]
   │
 3 │ for i = 10, 1 do
   │         ──┬──  
   │           ╰──── Do you mean `10, 1, -1`?
   │ 
   │ Help: for further information visit https://luals.github.io/wiki/diagnostics/#count-down-loop
───╯
[999] Error: Invalid global (`_ENV` is `nil`) (global-in-nil-env)
   ╭─[bad_code.lua:1:1]
   │
 1 │ _ENV = nil
   │ ──┬─  
   │   ╰─── Assignment occurs here
   │ 
   │ Help: for further information visit https://luals.github.io/wiki/diagnostics/#global-in-nil-env
───╯
[999] Error: Global variable `foo` starts with a lowercase letter (lowercase-global)
   ╭─[bad_code.lua:6:1]
   │
 6 │ foo = { i = 1 }
   │ ─┬─  
   │  ╰─── Dis you miss `local` or misspell it?
   │ 
   │ Help: for further information visit https://luals.github.io/wiki/diagnostics/#lowercase-global
───╯
lunalint: error: exited with 1 due to previous errors
```
