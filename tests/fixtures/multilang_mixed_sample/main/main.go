package main

import (
	"fmt"
	"os"
	"github.com/example/multilang-mixed/domain"
	"github.com/example/multilang-mixed/usecase"
	"github.com/example/multilang-mixed/infrastructure"
)

func main() {
	repo := &infrastructure.UserRepo{}
	uc := &usecase.UserUsecase{}
	user := domain.NewUser("John")
	_ = repo
	_ = uc
	fmt.Fprintln(os.Stdout, "user:", user)
}
