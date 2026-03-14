package com.example.myapp.usecase;

import com.example.myapp.domain.User;

/**
 * UserService handles business logic for users.
 */
public class UserService {
    public User createUser(int id, String name) {
        return new User(id, name);
    }
}
