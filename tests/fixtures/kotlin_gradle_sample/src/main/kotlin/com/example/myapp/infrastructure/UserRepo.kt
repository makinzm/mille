package com.example.kotlinapp.infrastructure

import com.example.kotlinapp.domain.User

class UserRepo {
    fun findUser(id: Int): User = User(id, "Bob")
    fun findAll(): List<User> = listOf(User(1, "Alice"), User(2, "Bob"))
}
