from dataclasses import dataclass
from typing import TypedDict


@dataclass
class Point:
    x: int
    y: int


class Row(TypedDict):
    id: int


class Marker:
    kind: str
