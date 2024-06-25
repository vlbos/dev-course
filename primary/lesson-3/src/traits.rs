trait Fruit {
    fn name() -> String {
        "Apple".into()
    }

    fn price() -> u32 {
        0
    }
}

struct Apple {
    pub name: String,
    pub price: u64,
}

impl Fruit for Apple {
    fn name() -> String {
        todo!()
    }
    fn price() -> u32 {
        todo!()
    }
}

// avoid the cycle reference

// decouple interface and implementation

// type check
