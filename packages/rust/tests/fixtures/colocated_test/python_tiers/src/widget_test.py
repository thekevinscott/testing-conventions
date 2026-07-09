from widget import greet


def describe_widget():
    def it_greets():
        assert greet("Ada") == "Hello, Ada!"
