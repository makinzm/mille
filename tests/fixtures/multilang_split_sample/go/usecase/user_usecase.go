package usecase

import "github.com/example/multilang-split/go/domain"

// UserUsecase handles user business logic.
type UserUsecase struct{}

// GetUser returns a user by name.
func (u *UserUsecase) GetUser(name string) *domain.User {
	return &domain.User{Name: name}
}
