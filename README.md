# mangler
A simple Rust program that mangles and parse C++ symbol names using Itanium ABI.

## Example
```rust
use mangler::*;

fn main() {
    let mangled = mangle("foo::bar::hello(int, float, std::string)");
    println!("{}", mangled) // _ZN3foo3bar5helloEifSs

    let mangled = mangle(
        Symbol::Function(
            Box::new(
                Symbol::Namespace(
                    vec![
                        Symbol::Type("hello".to_string()),
                        Symbol::Type("world".to_string()),
                        Symbol::Generic(
                            vec![
                                Symbol::Typed(
                                    Box::new(
                                        Symbol::Type("int".to_string())
                                    ),
                                    vec![TypedElement::Ptr]
                                )
                            ]
                        ),
                        Symbol::Type("print".to_string())
                    ]
                )
            ),
            vec![
                Symbol::Type("int".to_string()),
                Symbol::Type("float".to_string()),
                Symbol::Type("std::string".to_string())
            ]
        )
    );

    println!("{}", mangled); // _ZN5hello5worldIPiE5printEifSs
  
}
```
