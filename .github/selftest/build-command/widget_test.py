from widget import shift


def test_shift_positive():
    assert shift(1) == 11


def test_shift_zero():
    assert shift(0) == 10


def test_shift_negative():
    assert shift(-5) == 5
