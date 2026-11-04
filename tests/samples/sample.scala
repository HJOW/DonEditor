package demo

import scala.collection.mutable

// 한 줄 주석
trait Shape {
  def area: Double
}

case class Circle(radius: Double) extends Shape {
  override def area: Double = math.Pi * radius * radius
}

object Main {
  def main(args: Array[String]): Unit = {
    val shapes: List[Shape] = List(Circle(2.0), Circle(3.5))
    val map = mutable.Map("키" -> 1)
    for (shape <- shapes if shape.area > 10) {
      println(s"넓이: ${shape.area}")
    }
    shapes.headOption match {
      case Some(Circle(r)) => println(r)
      case None            => println("없음")
    }
  }
}
