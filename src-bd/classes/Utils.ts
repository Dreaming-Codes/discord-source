import DiscordSourcePlugin from "../index";

export class Utils {
    /**
     * Waits for an element to be added to the DOM
     * @param selector The query selector of the element
     */
    static waitForElm(selector) {
        return new Promise(resolve => {
            if (document.querySelector(selector)) {
                return resolve(document.querySelector(selector));
            }

            const observer = new MutationObserver(() => {
                if (document.querySelector(selector)) {
                    resolve(document.querySelector(selector));
                    observer.disconnect();
                }
            });

            observer.observe(document.body, {
                childList: true,
                subtree: true
            });
        });
    }

    /**
     * Waits for a confirmation modal to be accepted or rejected
     */
    static async asyncShowConfirmationModal(...args: Parameters<typeof BdApi.UI.showConfirmationModal>): Promise<boolean> {
        return new Promise(resolve => {
            const oldOnConfirm = args[2].onConfirm;
            const oldOnCancel = args[2].onCancel;

            args[2].onConfirm = () => {
                if(oldOnConfirm){
                    oldOnConfirm();
                }
                resolve(true);
            }

            args[2].onCancel = () => {
                if(oldOnCancel){
                    oldOnCancel();
                }
                resolve(false);
            }

            BdApi.UI.showConfirmationModal(...args);
        });
    }

    static log(...msg: any[]) {
        console.log(...Utils.logString(...msg));
    }

    static warn(...msg: any[]) {
        console.warn(...Utils.logString(...msg));
    }

    static error(...msg: any[]) {
        console.error(...Utils.logString(...msg));
    }

    private static logString(...msg: any[]) {
        return [`%c[${DiscordSourcePlugin.name}]`, 'color: #bada55', ...msg];
    }




}