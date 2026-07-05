from widget import Point


def test_shift_moves_the_point():
    assert Point(1, 2).shift(3, 4) == Point(4, 6)
