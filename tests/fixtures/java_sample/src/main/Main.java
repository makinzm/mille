package com.example.javasample.main;

import com.example.javasample.domain.User;
import com.example.javasample.usecase.UserService;
import com.example.javasample.infrastructure.UserRepo;

/**
 * Main is the application entry point — imports from all layers.
 */
public class Main {
    public static void main(String[] args) {
        UserRepo repo = new UserRepo();
        UserService service = new UserService();
        User user = service.createUser(1, "Alice");
        System.out.println(user.getName());
    }
}
