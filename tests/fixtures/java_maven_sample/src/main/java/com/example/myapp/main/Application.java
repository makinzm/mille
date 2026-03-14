package com.example.myapp.main;

import com.example.myapp.domain.User;
import com.example.myapp.usecase.UserService;
import com.example.myapp.infrastructure.UserRepo;

/**
 * Application is the entry point — imports from all layers.
 */
public class Application {
    public static void main(String[] args) {
        UserRepo repo = new UserRepo();
        UserService service = new UserService();
        User user = service.createUser(1, "Alice");
        System.out.println(user.getName());
    }
}
