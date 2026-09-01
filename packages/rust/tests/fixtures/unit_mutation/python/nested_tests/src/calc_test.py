from calc import add, is_positive


def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0


def test_is_positive():
    assert is_positive(1) is True
    assert is_positive(5) is True
    assert is_positive(-5) is False
    assert is_positive(0) is False
