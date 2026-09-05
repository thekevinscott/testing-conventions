_HERMETIC_CALLER = "thekevinscott/testing-conventions"
HERMETIC_CLI_COMMAND = "./hermetic-cli/testing-conventions"
HERMETIC_TS_ADAPTER_ARGS = "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js"


def hermetic(caller_repository: str, version: str) -> bool:
    """Whether this run gates the commit under test hermetically: this repository's own caller
    with no explicit `version` — an explicit version names the published artifact and wins, and
    every other caller takes the published path."""
    return caller_repository == _HERMETIC_CALLER and not version
