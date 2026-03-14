package com.example.kotlinsample.infrastructure

import com.example.kotlinsample.domain.User

/**
 * UserRepo is the infrastructure implementation for user storage.
 */
class UserRepo {
    fun findUser(id: Int): User = User(id, "Bob")
    fun findAll(): List<User> = listOf(User(1, "Alice"), User(2, "Bob"))
}
