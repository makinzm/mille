<?php
namespace App\Domain;

class User {
    public string $name;
    public string $email;

    public function __construct(string $name, string $email) {
        $this->name = $name;
        $this->email = $email;
    }

    public static function create(string $name, string $email): self {
        return new self($name, $email);
    }
}
