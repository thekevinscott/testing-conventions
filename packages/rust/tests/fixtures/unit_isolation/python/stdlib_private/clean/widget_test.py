from __future__ import annotations

import _ast
import _collections_abc
import _socket
import _thread

from myproject.widget import build


def describe_build():
    def it_builds():
        assert build(_ast, _collections_abc, _socket, _thread) is not None
