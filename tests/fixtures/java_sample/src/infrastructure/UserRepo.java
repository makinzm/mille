package com.example.javasample.infrastructure;

import com.example.javasample.domain.User;
import java.util.List;

/**
 * UserRepo is the infrastructure implementation for user storage.
 */
public class UserRepo {
    public User findUser(int id) {
        return new User(id, "Bob");
    }

    public List<User> findAll() {
        return List.of(new User(1, "Alice"), new User(2, "Bob"));
    }
}
