use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x = verifier::any!();
    verifier::assume!(x < i32::MAX/2);
    let res = iter_mut(x);
    if x >= 0 {
        verifier::vassert!(res == x);
    } else {
        verifier::vassert!(res == x*2);
    }
}

#[no_mangle]
pub extern "C" fn iter_mut(input: i32) -> i32 {
    if input >= 0 {
        let mut x: Result<i32, &str> = Ok(input);
        match x.iter_mut().next() {
            Some(v) => *v = input,
            None => {},
        };
        let y: Result<i32, i32> = Err(x.unwrap());
        match y {
            Ok(value) => value,
            Err(err) => err,
        }
    } else {
        let y: Result<i32, i32> = Err(input);
        let mut x: Result<i32, &str> = Ok(match y {
            Ok(value) => value,
            Err(err) => err,
        });
        
        match x.iter_mut().next() {
            Some(v) => *v = input,
            None => {},
        };
        x.unwrap()*2
    }

}

// #[no_mangle]
// #[inline(never)]
// pub extern "C" fn sea_nd_foo() -> i32 {
//     0
// }

