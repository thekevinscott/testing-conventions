from widget import classify


def test_pos():
    assert classify(1) == "pos"


def test_nonpos():
    assert classify(-1) == "nonpos"
