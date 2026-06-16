from widget import clamp


def test_clamp_caps_at_the_max():
    assert clamp(10) == 3


def test_clamp_passes_small_values_through():
    assert clamp(1) == 1
