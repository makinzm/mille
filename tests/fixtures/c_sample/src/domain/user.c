#include "user.h"
#include <string.h>

User user_create(const char *name, const char *email) {
    User u;
    strncpy(u.name, name, sizeof(u.name) - 1);
    u.name[sizeof(u.name) - 1] = '\0';
    strncpy(u.email, email, sizeof(u.email) - 1);
    u.email[sizeof(u.email) - 1] = '\0';
    return u;
}
