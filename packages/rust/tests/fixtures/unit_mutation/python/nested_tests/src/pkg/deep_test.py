from pkg.deep import is_negative, total


def test_total():
    assert total(2, 3) == 5
    assert total(-1, 1) == 0


def test_is_negative():
    assert is_negative(-1) is True
    assert is_negative(-5) is True
    assert is_negative(1) is False
    assert is_negative(0) is False
