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
	user := uc.GetUser(1)
	_ = repo
	_ = user
	fmt.Fprintln(os.Stdout, "user:", domain.User{})
}
