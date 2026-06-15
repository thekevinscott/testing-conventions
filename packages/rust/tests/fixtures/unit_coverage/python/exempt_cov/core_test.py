from core import classify


def test_positive():
    assert classify(1) == "positive"


def test_nonpositive():
    assert classify(-1) == "nonpositive"
