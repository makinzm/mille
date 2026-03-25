package main

import (
	"fmt"
	"os"
	"github.com/example/multilang-split/go/domain"
	"github.com/example/multilang-split/go/usecase"
	"github.com/example/multilang-split/go/infrastructure"
)

func main() {
	repo := &infrastructure.UserRepo{}
	uc := &usecase.UserUsecase{}
	user := domain.NewUser("John")
	_ = repo
	_ = uc
	fmt.Fprintln(os.Stdout, "user:", user)
}
