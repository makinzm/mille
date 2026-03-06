import { User } from "@/domain/user";

export function createUser(name: string): User {
  return new User(name);
}
