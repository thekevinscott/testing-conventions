from widget import classify


def test_positive():
    assert classify(1) == "positive"


def test_negative():
    assert classify(-1) == "negative"


def test_zero():
    assert classify(0) == "zero"
