import ast
import json
import tomllib

from ast import parse as parse_ast
from json import loads as load_json
from tomllib import loads as load_toml

ast.parse("x = 1")
json.loads("{}")
tomllib.loads("a = 1")
parse_ast("x = 1")
load_json("{}")
load_toml("a = 1")
