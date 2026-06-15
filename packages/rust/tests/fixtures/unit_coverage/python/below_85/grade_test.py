from grade import grade


def test_a():
    assert grade(95) == "A"


def test_f():
    assert grade(50) == "F"
