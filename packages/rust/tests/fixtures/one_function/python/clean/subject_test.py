from subject import ratio


def test_halves():
    result = ratio(2, 4)
    assert result == 3


def test_floors():
    result = ratio(1, 2)
    assert result == 1


def test_zero():
    result = ratio(0, 0)
    assert result == 0
