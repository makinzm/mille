package com.example.kotlinsample.usecase

import com.example.kotlinsample.domain.User

/**
 * UserService handles business logic for users.
 */
class UserService {
    fun createUser(id: Int, name: String): User = User(id, name)
}
