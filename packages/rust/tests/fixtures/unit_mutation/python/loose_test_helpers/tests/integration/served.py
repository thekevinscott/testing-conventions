def page_url(port, path):
    if path.startswith("/"):
        return f"http://localhost:{port}{path}"
    return f"http://localhost:{port}/{path}"
