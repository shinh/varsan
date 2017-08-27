extern crate goblin;
extern crate memmap;
extern crate owning_ref;

use log;
use self::goblin::elf;
use self::owning_ref::OwningHandle;

pub struct Symbol<'a> {
    pub name: &'a str,
    pub value: u64,
}

pub struct Binary<'a> {
    filename: String,
    o: OwningHandle<Box<memmap::Mmap>, Box<goblin::elf::Elf<'a>>>,
    bias: u64,
}

impl<'a> Binary<'a> {
    pub fn new(filename: String) -> Result<Self, String> {
        log_info!("Reading {}...", filename);
        let mem = try!(
            memmap::Mmap::open_path(&filename, memmap::Protection::Read)
                .or(Err(format!("Failed to open: {}", &filename))));
        let o = try!(OwningHandle::try_new(
            Box::new(mem), |mem| -> Result<_, _> {
                let mem = unsafe { &*mem };
                match elf::Elf::parse(unsafe { mem.as_slice() }) {
                    Ok(file) => Ok(Box::new(file)),
                    Err(_) => Err(format!("Parse failed: {}", &filename)),
                }
            }));
        return Ok(Binary {
            filename: filename,
            o: o,
            bias: 0,
        });
    }

    pub fn filename(&self) -> &String { &self.filename }

    pub fn syms(&self) -> Vec<Symbol<'a>> {
        let (syms, strtab) = if self.o.syms.len() == 0 {
            (&self.o.dynsyms, &self.o.dynstrtab)
        } else {
            (&self.o.syms, &self.o.strtab)
        };

        let mut r = vec!();
        for sym in syms {
            if sym.st_name == 0 {
                continue;
            }
            match strtab.get(sym.st_name) {
                Ok(name) => {
                    r.push(Symbol {
                        name: name,
                        value: sym.st_value as u64
                    });
                }
                Err(_) => {
                    println!("{}: invalid strtab", self.filename);
                }
            }
        }
        return r;
    }

    pub fn interp(&self) -> Option<&str> {
        return self.o.interpreter;
    }

    pub fn entry(&self) -> u64 {
        return self.o.entry;
    }

    #[allow(dead_code)]
    pub fn bias(&self) -> u64 {
        return self.bias;
    }

    pub fn set_bias(&mut self, bias: u64) {
        self.bias = bias;
    }
}

#[test]
fn test_c_binary() {
    let bin = Binary::new("test/data/hello".to_string()).unwrap();
    let mut found_count = 0;
    for sym in bin.syms() {
        if sym.name == "main" {
            assert_eq!(0x4005d0, sym.value);
            found_count += 1;
        }
    }
    assert_eq!(1, found_count);
}
