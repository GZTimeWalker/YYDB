use yengine::*;

#[test]
#[cfg(test)]
fn test_ffi() {
    let a = 1;
    let b = 2;

    println!("{} + {} = {} -- from rust", a, b, a + b);
    ffi::do_test();
}
