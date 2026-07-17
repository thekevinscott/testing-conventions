from widget import shout


def test_shout_uses_the_package_root_fixture(greeting):
    assert shout(greeting) == "hello!"
