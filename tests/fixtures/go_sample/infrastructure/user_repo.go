package infrastructure

import (
	"database/sql"
	"github.com/example/gosample/domain"
)

// UserRepo is the infrastructure implementation for user storage.
type UserRepo struct {
	db *sql.DB
}

// FindUser retrieves a user from the database.
func (r *UserRepo) FindUser(id int) *domain.User {
	return &domain.User{ID: id, Name: "Bob"}
}
