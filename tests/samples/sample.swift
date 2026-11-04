import Foundation

// 한 줄 주석
protocol Shape {
    func area() -> Double
}

struct Circle: Shape {
    let radius: Double
    func area() -> Double {
        return Double.pi * radius * radius
    }
}

class Counter {
    private var count = 0
    func increment(by step: Int = 1) -> Int {
        count += step
        return count
    }
}

enum Level: String {
    case low, high
}

let shapes: [Shape] = [Circle(radius: 2.0)]
for shape in shapes {
    if shape.area() > 10 {
        print("넓이: \(shape.area())")
    }
}
guard let value = Int("42") else { fatalError("실패") }
