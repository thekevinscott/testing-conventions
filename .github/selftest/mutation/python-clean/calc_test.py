from calc import double


def test_doubles_a_positive():
    assert double(3) == 6


def test_doubles_a_negative():
    assert double(-4) == -8


def test_doubles_zero():
    assert double(0) == 0
