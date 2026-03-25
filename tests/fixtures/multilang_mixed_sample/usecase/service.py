"""Usecase layer — may depend on domain only."""
from domain.entity import User


class UserService:
    def get_user(self, name: str) -> User:
        return User(name)
