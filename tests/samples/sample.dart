import 'dart:async';

// 한 줄 주석
abstract class Shape {
  double area();
}

class Circle extends Shape {
  final double radius;
  Circle(this.radius);

  @override
  double area() => 3.14159 * radius * radius;
}

Future<void> main() async {
  final shapes = <Shape>[Circle(2), Circle(3.5)];
  for (final shape in shapes) {
    if (shape.area() > 10 && shape != null) {
      print('넓이: ${shape.area()}');
    }
  }
  var name = "Don Editor";
  await Future.delayed(const Duration(milliseconds: 10));
  print(name);
}
