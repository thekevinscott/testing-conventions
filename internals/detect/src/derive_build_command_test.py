import derive_build_command


def test_derive_build_command_reads_the_language_table(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert derive_build_command.derive_build_command("tc.toml", "python") == "make protos"


def test_derive_build_command_is_empty_for_another_language(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert derive_build_command.derive_build_command("tc.toml", "typescript") == ""


def test_derive_build_command_is_empty_without_a_language(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert derive_build_command.derive_build_command("tc.toml", "") == ""


def test_derive_build_command_is_empty_when_the_config_is_absent(tmp_path, in_dir):
    in_dir(tmp_path)
    assert derive_build_command.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_is_empty_when_the_config_is_malformed(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", "[python\n")
    assert derive_build_command.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_ignores_a_non_string_declaration(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = ["make", "protos"]\n')
    assert derive_build_command.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_keeps_a_multiline_declaration(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = """\ncp a.tmpl a.py\ncp b.tmpl b.py\n"""\n')
    derived = derive_build_command.derive_build_command("tc.toml", "python")
    assert derived == "cp a.tmpl a.py\ncp b.tmpl b.py\n"
