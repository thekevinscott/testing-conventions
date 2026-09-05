import e2e_scope_flags


def test_derive_e2e_extra_scope_renders_repeated_flags(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = ["core", "shared/proto"]\n')
    derived = e2e_scope_flags.derive_e2e_extra_scope("tc.toml")
    assert derived == "--extra-scope core --extra-scope shared/proto"


def test_derive_e2e_exclude_renders_repeated_flags(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nexclude = ["core/cli"]\n')
    assert e2e_scope_flags.derive_e2e_exclude("tc.toml") == "--exclude core/cli"


def test_e2e_scope_flags_is_empty_when_the_config_is_absent(tmp_path, in_dir):
    in_dir(tmp_path)
    assert e2e_scope_flags.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_when_the_config_is_malformed(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", "[e2e\n")
    assert e2e_scope_flags.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_when_the_key_is_absent(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nexclude = ["core/cli"]\n')
    assert e2e_scope_flags.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_for_a_non_list_declaration(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = "core"\n')
    assert e2e_scope_flags.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_drops_non_string_and_empty_entries(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = ["core", "", 7]\n')
    assert e2e_scope_flags.derive_e2e_extra_scope("tc.toml") == "--extra-scope core"
