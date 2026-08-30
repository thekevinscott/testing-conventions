class Widget:
    def __init__(self, size):
        self.size = size

    def grow(self, amount):
        self.size += amount
        return self.size

    def shrink(self, amount):
        self.size -= amount
        return self.size
