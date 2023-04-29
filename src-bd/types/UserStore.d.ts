import {User} from "./User";

export interface UserStore {
    getUser: (id: string) => User;
}