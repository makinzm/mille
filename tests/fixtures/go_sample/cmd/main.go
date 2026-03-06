package main

import (
	"fmt"
	"os"
	"github.com/example/gosample/domain"
	"github.com/example/gosample/usecase"
	"github.com/example/gosample/infrastructure"
)

func main() {
	repo := &infrastructure.UserRepo{}
	uc := &usecase.UserUsecase{}
	user := domain.NewUser("John")
	_ = repo
	_ = uc
	fmt.Fprintln(os.Stdout, "user:", user)
	_ = os.Stdout
}
