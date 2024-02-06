fn write(_value: *const std::os::raw::c_char) { }

custom_print::define_macros!({ cprint, cprintln, ceprint, ceprintln }, concat, crate::write);
macro_rules! print { ($($args:tt)*) => { cprint!($($args)*); } }
// macro_rules! println { ($($args:tt)*) => { cprintln!($($args)*); } }
// macro_rules! eprint { ($($args:tt)*) => { ceprint!($($args)*); } }
// macro_rules! eprintln { ($($args:tt)*) => { ceprintln!($($args)*); } }

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    print!("test");
    // println!("test");
    // print!("test {}", 42);
    // println!("test {}", 42);
    // eprint!("test");
    // eprintln!("test");
    // eprint!("test {}", 42);
    // eprintln!("test {}", 42);
}
