## The Uma Programming Language

Uma is a toy programming language built with Rust. Initially, it started as a project to learn Rust, but I fell into the rabbit hole of lang-dev. As it is now, it parses the source into an AST and transpiles to C under the hood.

## Installation

Uma can be installed directly via cargo:

```sh
$ cargo install --git https://github.com/du-cki/Uma
```

## Features

Even as a toy language, Uma supports a foundation of standard programming constructs:

- [x] Variables & Types
- [x] If/Else If/Else control flows
- [x] Ranged Iterations
- [x] Functions
- [x] C Bindings (via `@requires`)
- [ ] Arrays
- [ ] Structs

## Example

```go
func printf(format, ...) @requires("stdio.h")

func main(): int {
    printf("Hello, World!");
    return 0;
}
```

More examples can be found in the [`examples/`](examples/) directory in the repository.
