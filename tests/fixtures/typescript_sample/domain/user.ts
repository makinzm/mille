export class User {
    constructor(public name: string, public id: number) {}

    static create(name: string): User {
        return new User(name, 0);
    }
}
