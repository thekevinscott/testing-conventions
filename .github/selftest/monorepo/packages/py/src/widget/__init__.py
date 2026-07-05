import attr


@attr.s(auto_attribs=True)
class Point:
    x: int
    y: int

    def shift(self, dx: int, dy: int) -> "Point":
        return Point(self.x + dx, self.y + dy)
