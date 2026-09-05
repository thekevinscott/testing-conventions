from . import helper


class Client:
    def send(self, payload):
        return helper.run(payload)
