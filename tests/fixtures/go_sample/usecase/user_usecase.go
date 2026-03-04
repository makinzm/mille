package usecase

import "github.com/example/gosample/domain"

// UserUsecase handles user business logic.
type UserUsecase struct{}

// GetUser returns a user by ID.
func (u *UserUsecase) GetUser(id int) *domain.User {
	return &domain.User{ID: id, Name: "Alice"}
}
