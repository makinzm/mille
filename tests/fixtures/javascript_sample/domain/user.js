export class User {
    constructor(name, id) {
        this.name = name;
        this.id = id;
    }

    static create(name) {
        return new User(name, 0);
    }
}
