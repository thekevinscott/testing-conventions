from widget import classify


def test_classify():
    assert classify(1) == "positive"
    assert classify(-1) == "non-positive"
