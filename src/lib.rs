
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct CxxParser;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Namespace(Vec<Symbol>),
    Function(Box<Symbol>, Vec<Symbol>),
    Constructor(Vec<Symbol>),
    Generic(Vec<Symbol>),
    Type(String),
    Operator(String),
    Typed(Box<Symbol>, Vec<TypedElement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedElement {
    Ref,
    Ptr,
    Const,
}

impl Symbol {

    fn parse_pair(pair: Pair<Rule>) -> Symbol {
        match pair.as_rule() {
            Rule::namespace => {
                let mut namespace = Vec::new();
                for pair in pair.into_inner() {
                    namespace.push(Symbol::parse_pair(pair));
                }
                Symbol::Namespace(namespace)
            },
            Rule::function => {
                let mut pairs = pair.into_inner();
                let func = pairs.next().unwrap();
                let func = Symbol::parse_pair(func);
                let mut args = Vec::new();
                let is_const = if let Some(p) = pairs.clone().last() {
                    p.as_rule() == Rule::const_
                } else {
                    false
                };
                for pair in pairs.clone().take(if is_const { pairs.len() - 1 } else { pairs.len() }) {
                    args.push(Symbol::parse_pair(pair));
                }
                let func = Symbol::Function(Box::new(func), args);
                if is_const {
                    return Symbol::Typed(Box::new(func), vec![TypedElement::Const])
                }
                func

            },
            Rule::generic => {
                let mut generic = Vec::new();
                for pair in pair.into_inner() {
                    generic.push(Symbol::parse_pair(pair));
                }
                Symbol::Generic(generic)
            },
            Rule::type_ => {
                let inners = pair.into_inner();
                Symbol::Type(inners.map(|x| x.as_str().to_string()).collect::<Vec<String>>().join(" "))
            },
            Rule::element => {
                Symbol::parse_pair(pair.into_inner().next().unwrap())
            },
            Rule::ty_element => {
                let mut pairs = pair.into_inner();
                let element = Symbol::parse_pair(pairs.next().unwrap());
                let mut typed_element = Vec::new();
                for p in pairs {
                    typed_element.push(
                        match p.as_rule() {
                            Rule::const_ => TypedElement::Const,
                            Rule::ptr => TypedElement::Ptr,
                            Rule::ref_ => TypedElement::Ref,
                            _ => unreachable!(),
                        }
                    )
                }
                Symbol::Typed(Box::new(element), typed_element)
            },
            e => panic!("Invalid symbol: {:?}", e)
        }
    }

    pub fn parse(s: &str) -> Symbol {
        let pairs =  <CxxParser as Parser<Rule>>::parse(Rule::function, s).unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.clone().next().unwrap();
        Symbol::parse_pair(pair)
    }
}


#[derive(Debug, Clone)]
pub struct Mangler {
    used_namespace: Vec<Symbol>,
    should_be_const: bool
}

impl Mangler {
    fn new() -> Self {
        Mangler {
            used_namespace: Vec::new(),
            should_be_const: false
        }
    }
    fn mangle(&mut self, symbol: Symbol) -> String {
        match symbol {
            Symbol::Namespace(n) => {
                self.mangle_namespace(n)
            },
            Symbol::Type(n) => {
                self.mangle_type(n)
            }
            Symbol::Generic(gen) => {
                self.mangle_generic(gen)
            },
            Symbol::Function(
                c, a
            ) => {
                self.mangle_function(*c, a)
            },
            Symbol::Typed(s, te) => {
                let mut string = String::new();
                if let Symbol::Function(..) = *s {
                    self.should_be_const = true;
                    return self.mangle(*s);
                }
                for e in te {
                    string.push(match e {
                        TypedElement::Ptr => 'P',
                        TypedElement::Const => 'K',
                        TypedElement::Ref => 'R'
                    })
                }

                string.push_str(&self.mangle(*s));


                string
            }
            e => panic!("Invalid symbol: {:?}", e)
        }
    }

    fn mangle_function(&mut self, symbol: Symbol, syms: Vec<Symbol>) -> String {
        let mut s = self.mangle(symbol);
        for sym in syms {
            s.push_str(&self.mangle(sym));
        }
        s
    }

    fn mangle_generic(&mut self, syms: Vec<Symbol>) -> String {
        let mut mangled = String::new();
        mangled.push('I');
        for symbol in syms {
            mangled.push_str(&self.mangle(symbol.clone()));
        }
        mangled.push('E');
        mangled
    }

    fn mangle_type(&mut self, ty: String) -> String {
        let mut s = String::new();
        let binding = ty.len().to_string() + &ty;
        s.push_str(match ty.as_str() {
            "std::string" => "Ss",
            "schar" => "a",
            "bool" => "b",
            "char" => "c",
            "double" => "d",
            "long double" => "e",
            "float" => "f",
            "__float128" => "g",
            "unsigned char" => "h",
            "int" => "i",
            "unsigned int" => "j",
            "long" => "l",
            "unsigned long" => "m",
            "__int128" => "n",
            "unsigned __int128" => "o",
            "short" => "s",
            "std::allocator" => "Sa",
            "std::basic_string" => "Sb",
            "std::basic_iostream<char, std::char_traits<char>>" => "Sd",
            "std::basic_istream<char, std::char_traits<char>>" => "Si",
            "std::basic_ostream<char, std::char_traits<char>>" => "So",
            "std::basic_string<char, std::char_traits<char>, std::allocator<char>>" => "Ss",
            "std" => "St",
            "unsigned short" => "t",
            "void" => "v",
            "volatile" => "V",
            "wchar_t" => "w",
            "long long" => "x",
            "unsigned long long" => "y",
            "ellipsis" => "z",
            _ => binding.as_str()
        });

        s
    }

    fn mangle_namespace(&mut self, namespace: Vec<Symbol>) -> String {
        let mut mangled = String::from("N");
        if self.should_be_const {
            mangled.push('K');
            self.should_be_const = false;
        }

        for symbol in namespace.iter() {
            if self.used_namespace.contains(symbol) {
                mangled.push_str("S_")
            } else {
                self.used_namespace.push(symbol.clone());
                mangled.push_str(&self.mangle(symbol.clone()));
            }
        }

        mangled.push('E');
        mangled
    }
}

pub fn mangle(s: String) -> String {
    let symbol = Symbol::parse(&s);
    let mut mangled = Mangler::new();
    "_Z".to_string() + &mangled.mangle(symbol)
}
pub fn mangle_symbol(s: Symbol) -> String {
    let mut mangled = Mangler::new();
    "_Z".to_string() + &mangled.mangle(s)
}
