from calc import TOTALS, add


def test_add():
    assert add(2, 3) == 5
    assert add(2) == 8
    assert add(None, 3) is None


def test_totals():
    assert TOTALS == {"total": 7}
