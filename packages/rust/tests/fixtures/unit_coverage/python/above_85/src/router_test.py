from router import route


def test_home():
    assert route("GET", "/") == "home"


def test_page():
    assert route("GET", "/other") == "page"


def test_create():
    assert route("POST", "/") == "create"
