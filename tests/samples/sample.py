# 한 줄 주석
import os
from dataclasses import dataclass


@dataclass
class Point:
    """독스트링 주석"""
    x: float
    y: float = 0.0

    def distance(self) -> float:
        return (self.x ** 2 + self.y ** 2) ** 0.5


def main(argv: list[str]) -> int:
    name = f"Don Editor {len(argv)}"
    points = [Point(1, 2), Point(3.5)]
    for p in points:
        if p.distance() > 1 and name is not None:
            print(name, p, os.sep)
    try:
        value = int("42")
    except ValueError as e:
        raise RuntimeError("실패") from e
    return 0


if __name__ == "__main__":
    raise SystemExit(main([]))
