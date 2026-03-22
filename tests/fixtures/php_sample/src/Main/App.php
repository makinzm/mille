<?php
namespace App\Main;

use App\Domain\User;
use App\UseCase\CreateUser;
use App\Infrastructure\UserRepo;

class App {
    public function run(): void {
        $user = User::create('Alice', 'alice@example.com');
        $useCase = new CreateUser();
        $repo = new UserRepo(new \PDO('sqlite::memory:'));
    }
}
