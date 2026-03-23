#include "../domain/user.h"
#include "../usecase/create_user.h"
#include "../infrastructure/user_repo.h"
#include <stdio.h>

int main(void) {
    User user = create_user("Alice", "alice@example.com");
    user_repo_save(&user);
    printf("Done\n");
    return 0;
}
