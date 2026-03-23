#include "create_user.h"

User create_user(const char *name, const char *email) {
    return user_create(name, email);
}
