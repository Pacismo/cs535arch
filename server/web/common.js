/**
 * @param {(container: HTMLElement, close: () => void) => void} populator A callback that populates the `container`
 *     with HTML objects
 */
export function popup(populator) {
    let body = document.getElementsByTagName("body").item(0);

    /** @type {HTMLDivElement} */
    let popup = document.createElement("div");
    popup.classList = "popup";
    body.appendChild(popup);

    /** @type {HTMLDivElement} */
    let container = document.createElement("div");
    popup.appendChild(container);

    populator(container, () => body.removeChild(popup));
}

/** @type {(container: HTMLElement) => (pair: [string, any]) => void} */
export function put_into(container) {
    return pair => {
        let key = pair[0];
        let value = pair[1];

        let list_item = document.createElement('li');
        let k = document.createElement('b');
        k.textContent = `${key}: `;
        list_item.appendChild(k);
        let v = null;

        switch (typeof value) {
        case 'bigint':
        case 'boolean':
        case 'number':
        case 'string':
            v = document.createTextNode(`${value}`);
            break;
        case 'object':
            v = document.createElement('ul');
            Object.entries(value).forEach(put_into(v));
            break;
        default:
            v = document.createTextNode('???');
            break;
        }

        list_item.appendChild(v);
        container.appendChild(list_item);
    };
}
