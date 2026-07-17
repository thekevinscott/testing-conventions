def route(method, path):
    if method == "GET":
        if path == "/":
            return "home"
        return "page"
    if method == "POST":
        return "create"
    return "not allowed"
