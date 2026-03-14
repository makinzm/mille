package com.example.kotlinapp.main

import com.example.kotlinapp.domain.User
import com.example.kotlinapp.usecase.UserService
import com.example.kotlinapp.infrastructure.UserRepo

fun main() {
    val service = UserService()
    val repo = UserRepo()
    println(service.createUser(1, "Alice"))
}
