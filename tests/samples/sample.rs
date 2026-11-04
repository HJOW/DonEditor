use std::collections::HashMap;

/// 문서 주석
#[derive(Debug, Clone)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T: Copy + std::ops::Add<Output = T>> Point<T> {
    fn sum(&self) -> T {
        self.x + self.y
    }
}

enum Shape {
    Circle(f64),
    Square { side: f64 },
}

fn main() {
    // 한 줄 주석
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("키".to_string(), 42);
    let p = Point { x: 1, y: 2 };
    println!("합계: {}", p.sum());
    for (key, value) in &map {
        if *value > 10 {
            println!("{key} = {value}");
        }
    }
    let shape = Shape::Square { side: 3.0 };
    match shape {
        Shape::Circle(r) => println!("{}", r),
        Shape::Square { side } => println!("{}", side),
    }
}
