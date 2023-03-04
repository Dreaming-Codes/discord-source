export class SharedUtils {
    static delay(ms: number) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}