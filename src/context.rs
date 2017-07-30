use binary;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: binary::Binary<'a>,
    symtab: HashMap<&'a str, u64>,
}

impl<'a> Context<'a> {
    pub fn new(main_binary: &str) -> Result<Self, String> {
        let bin = try!(binary::Binary::new(main_binary.to_string()));
        let mut symtab = HashMap::new();
        for sym in bin.syms() {
            symtab.insert(sym.name, sym.value);
        }
        Ok(Self {
            main_binary: bin,
            symtab: symtab,
        })
    }

    pub fn resolve(&self, name: &str) -> Option<&u64> {
        return self.symtab.get(name);
    }
}
