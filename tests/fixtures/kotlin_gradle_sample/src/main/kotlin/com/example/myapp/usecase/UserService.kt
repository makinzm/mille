package com.example.kotlinapp.usecase

import com.example.kotlinapp.domain.User

class UserService {
    fun createUser(id: Int, name: String): User = User(id, name)
}
