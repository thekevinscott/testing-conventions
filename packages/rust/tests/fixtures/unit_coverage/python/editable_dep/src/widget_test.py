import a_thing

from widget import widget_fn


def test_widget():
    assert widget_fn(1) == "positive"
    assert widget_fn(-1) == "non-positive"


def test_a_thing_partially():
    assert a_thing.classify(1) == "positive"
