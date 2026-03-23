#include "user_repo.h"
#include <stdio.h>

int user_repo_save(const User *user) {
    // save user to database
    printf("Saving user: %s\n", user->name);
    return 0;
}
