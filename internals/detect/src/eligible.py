def eligible(languages_input: str, language: str) -> bool:
    """Whether `language` is in scope: an empty (or `[]`) `LANGUAGES` restrictor puts every
    supported language in scope; a non-empty JSON array restricts to the languages it names."""
    restrictor = languages_input.strip()
    return restrictor in ("", "[]") or f'"{language}"' in restrictor
