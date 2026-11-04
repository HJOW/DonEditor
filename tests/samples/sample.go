package main

import (
	"fmt"
	"strings"
)

// Shape 는 도형 인터페이스이다.
type Shape interface {
	Area() float64
}

type Circle struct {
	Radius float64
}

func (c Circle) Area() float64 {
	return 3.14159 * c.Radius * c.Radius
}

func main() {
	shapes := []Shape{Circle{Radius: 2}, Circle{Radius: 3.5}}
	for i, s := range shapes {
		if s.Area() > 10 {
			fmt.Println(i, strings.ToUpper("넓이"), s.Area())
		}
	}
	ch := make(chan int, 1)
	go func() { ch <- 42 }()
	fmt.Println(<-ch)
}
