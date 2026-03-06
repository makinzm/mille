import { User } from "../domain/user";
import * as fs from "node:fs";

export class UserRepository {
    private filePath: string;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    save(user: User): void {
        fs.writeFileSync(this.filePath, JSON.stringify(user));
    }
}
