#ifndef DOMAIN_USER_H
#define DOMAIN_USER_H

#include <string.h>

typedef struct {
    char name[64];
    char email[128];
} User;

User user_create(const char *name, const char *email);

#endif
