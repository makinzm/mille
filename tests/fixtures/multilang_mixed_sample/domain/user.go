package domain

// User is the core domain entity.
type User struct {
	ID   int
	Name string
}

// NewUser creates a new User.
func NewUser(name string) User {
	return User{Name: name}
}
