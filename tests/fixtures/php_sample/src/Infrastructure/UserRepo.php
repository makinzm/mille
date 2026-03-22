<?php
namespace App\Infrastructure;

use App\Domain\User;
use PDO;

class UserRepo {
    private PDO $pdo;

    public function __construct(PDO $pdo) {
        $this->pdo = $pdo;
    }

    public function save(User $user): void {
        // save to database
    }
}
