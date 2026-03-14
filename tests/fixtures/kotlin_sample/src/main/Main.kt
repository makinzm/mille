package com.example.kotlinsample.main

import com.example.kotlinsample.domain.User
import com.example.kotlinsample.usecase.UserService
import com.example.kotlinsample.infrastructure.UserRepo

fun main() {
    val service = UserService()
    val repo = UserRepo()
    val user = service.createUser(1, "Alice")
    println(user)
}
