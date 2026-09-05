import python_env


def test_python_env_is_uv_for_an_installable_project(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert python_env.python_env(tmp_path) == "uv"


def test_python_env_is_pip_for_a_tool_only_pyproject(tmp_path, write):
    write(tmp_path / "pyproject.toml", "[tool.ruff]\nline-length = 100\n")
    assert python_env.python_env(tmp_path) == "pip"


def test_python_env_is_pip_without_a_pyproject(tmp_path):
    assert python_env.python_env(tmp_path) == "pip"
