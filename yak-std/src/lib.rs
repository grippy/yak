#[no_mangle]
pub extern "C" fn print_int(x: i64) -> () {
    println!("printing: {}", x);
}

#[no_mangle]
pub extern "C" fn print_uint(x: u64) -> () {
    println!("printing: {}", x);
}

#[repr(C)]
#[derive(Debug)]
pub enum Person {
    Me,
    You,
}

#[no_mangle]
pub extern "C" fn print_enum(person: Person) -> () {
    println!("{:?}", &person)
}
