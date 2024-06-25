pub fn handle_option() {
    // Option
    let mut opt = Some(0_i32);

    match opt {
        Some(value) => println!("{}", value),
        None => println!("it is none"),
    }

    // Result
    let mut res = Ok(0_i32);
    match res {
        Ok(value) => println!("{}", value),
        Err(_) => println!("it is an error"),
    }

    // Option -> Result
    res = opt.ok_or("it is zero");
    println!("res is {:?}", res);

    // Result -> Option
    opt = res.ok();
    println!("opt is {:?}", opt);

    write_a_func().expect("message");

    // panic
    if opt.is_some() {
        panic!("panic happened.")
    }
}
pub fn write_a_func() -> Option<String> {
    unimplemented!()
}
