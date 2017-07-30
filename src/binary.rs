extern crate goblin;
extern crate memmap;
extern crate owning_ref;

use self::goblin::elf;
use self::owning_ref::OwningHandle;
use std;

pub struct Symbol<'a> {
    name: &'a str,
    value: u64,
}

pub struct Binary<'a> {
    filename: String,
    o: OwningHandle<Box<memmap::Mmap>, Box<goblin::elf::Elf<'a>>>,
}

impl<'a> Binary<'a> {
    pub fn new(filename: &str) -> Result<Binary, ()> {
        let mem = memmap::Mmap::open_path(
            filename, memmap::Protection::Read);
        if mem.is_err() {
            println!("Failed to open: {}", filename);
            return Err(());
        }
        let mem = mem.unwrap();
        let o = OwningHandle::try_new(Box::new(mem), |mem| -> Result<_, ()> {
            let mem = unsafe { &*mem };
            let file = elf::Elf::parse(unsafe { mem.as_slice() })
                .expect("Should parse object file");
            Ok(Box::new(file))
        }).unwrap();
        return Ok(Binary {
            filename: filename.to_string(),
            o: o,
        });
    }

    pub fn syms(&self) -> Vec<Symbol<'a>> {
        let syms = if self.o.syms.len() == 0 {
            &self.o.dynsyms
        } else {
            &self.o.syms
        };

        let mut r = vec!();
        for sym in syms {
            if sym.st_name == 0 {
                continue;
            }
            match self.o.strtab.get(sym.st_name) {
                Ok(name) => {
                    r.push(Symbol {
                        name: name,
                        value: sym.st_value as u64
                    });
                }
                Err(e) => {
                    println!("{}: invalid strtab", self.filename);
                }
            }
        }
        return r;
    }
}

#[test]
fn test_c_binary() {
    let bin = Binary::new("test/data/hello").unwrap();
    let mut found_count = 0;
    for sym in bin.syms() {
        if sym.name == "main" {
            assert_eq!(0x40051d, sym.value);
            found_count += 1;
        }
    }
    assert_eq!(1, found_count);
}
