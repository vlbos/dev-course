// struct
struct Point {
    x: i32,
    y: i32,
}
impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    fn print(&self) {
        println!("x: {}, y: {}", self.x, self.y);
    }
}

//enum
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn data_types() {
    // int
    let a = 0_i8;
    let b = 0_i16;
    let c = 0_i32;
    let d = 0_i64;
    let e = 0_i128;

    println!("{}, {}, {}, {}, {}", a, b, c, d, e);

    let f = 0_u8;
    let g = 0_u16;
    let h = 0_u32;
    let i = 0_u64;
    let j = 0_u128;

    println!("{}, {}, {}, {}, {}", f, g, h, i, j);

    let n = 0.0_f32;
    let o = 0.0_f64;

    let p = 0_usize;
    let q = 0_isize;

    let k = "Hello";
    let l = "Hello".to_string();
}
