import { User } from "../domain/model";
import { validate } from "some-lib";

export function createUser(name: string, id: number): User {
    validate(name);
    return new User(name, id);
}
