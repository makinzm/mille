import { User } from "../domain/user";
import fs from "node:fs";

export class UserRepository {
    constructor(filePath) {
        this.filePath = filePath;
    }

    save(user) {
        fs.writeFileSync(this.filePath, JSON.stringify(user));
    }
}
