from grade import grade


def test_high_score_passes():
    # The failing branch (`return "fail"`) is deliberately left uncovered, so the
    # suite lands below the floor and `unit coverage` must exit non-zero.
    assert grade(90) == "pass"
