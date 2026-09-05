import output_delimiter


def test_free_of_keeps_an_uncontested_base():
    assert output_delimiter.free_of("delim", ["one", "two"]) == "delim"


def test_free_of_bumps_past_a_colliding_line():
    assert output_delimiter.free_of("delim", ["delim"]) == "delim_1"


def test_free_of_bumps_until_the_suffix_is_free():
    assert output_delimiter.free_of("delim", ["delim", "delim_1", "delim_2"]) == "delim_3"


def test_output_delimiter_appears_on_no_line_of_the_value():
    value = "cp a.tmpl a.py\ncp b.tmpl b.py"
    assert output_delimiter.output_delimiter(value) not in value.split("\n")


def test_output_delimiter_is_deterministic_for_a_value():
    assert output_delimiter.output_delimiter("same") == output_delimiter.output_delimiter("same")


def test_output_delimiter_differs_between_values():
    assert output_delimiter.output_delimiter("one") != output_delimiter.output_delimiter("two")
