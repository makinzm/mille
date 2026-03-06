"""Domain entities — no internal dependencies allowed."""
import os


class User:
    def __init__(self, name: str) -> None:
        self.name = name
