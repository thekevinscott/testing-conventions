import eligible


def test_eligible_empty_restrictor_allows_any_language():
    assert eligible.eligible("", "python") is True


def test_eligible_empty_array_allows_any_language():
    assert eligible.eligible("[]", "rust") is True


def test_eligible_named_language_is_in_scope():
    assert eligible.eligible('["python"]', "python") is True


def test_eligible_unnamed_language_is_excluded():
    assert eligible.eligible('["python"]', "rust") is False
