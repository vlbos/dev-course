pub fn flows() {
    // if else
    let number = 0;
    if number > 0 {
        println!("number is positive")
    } else if number < 0 {
        println!("number is negative")
    } else {
        println!("number is zero")
    }

    // while loop
    let mut index = 0;
    while index < 10 {
        index += 1;
    }

    // for loop
    for index in 0..=10 {
        println!("index is {}", index);
    }

    // iterator
    let arr = [1, 2, 3, 4, 5];
    for index in arr.iter() {
        println!("index is {}", index);
    }

    // match
    match index {
        0..=10 => println!("index is in 0..10"),
        _ => println!("index is not in 0..10"),
    }

    // if let
    let opt: Option<i32> = Some(0);
    if let Some(0) = opt {
        println!("include a value, it is zero");
    }
}
