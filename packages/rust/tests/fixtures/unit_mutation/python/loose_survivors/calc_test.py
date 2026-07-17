from calc import add, is_positive


def test_add_runs():
    assert isinstance(add(2, 3), int)


def test_is_positive_runs():
    assert isinstance(is_positive(5), bool)
