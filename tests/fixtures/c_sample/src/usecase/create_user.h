#ifndef USECASE_CREATE_USER_H
#define USECASE_CREATE_USER_H

#include "../domain/user.h"

User create_user(const char *name, const char *email);

#endif
