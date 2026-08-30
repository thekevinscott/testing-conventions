def build(values):
    def inner(value):
        doubled = value * 2
        return doubled

    mapped = [inner(value) for value in values]
    return sorted(mapped, key=lambda item: -item)
