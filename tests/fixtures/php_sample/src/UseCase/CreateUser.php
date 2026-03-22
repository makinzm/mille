<?php
namespace App\UseCase;

use App\Domain\User;

class CreateUser {
    public function execute(string $name, string $email): User {
        return new User($name, $email);
    }
}
