/**
 * @param {(title: string, container: HTMLElement) => void} populator A callback that populates the `container`
 *     with HTML objects
 */
export function popup(title, populator) {
    let body = document.getElementsByTagName("body").item(0);

    /** @type {HTMLDivElement} */
    let popup = document.createElement("div");
    popup.classList = "popup";
    body.appendChild(popup);

    /** @type {HTMLDivElement} */
    let container = document.createElement("div");
    popup.appendChild(container);

    let pop_title = document.createElement('h2');
    pop_title.textContent = title;
    container.appendChild(pop_title);
    pop_title.style = 'margin: 0;';
    container.appendChild(document.createElement('hr'));

    let message = document.createElement('div');
    container.appendChild(message);

    container.appendChild(document.createElement('hr'));

    let exit_button = document.createElement('button');
    container.appendChild(exit_button);
    exit_button.onclick = () => body.removeChild(popup);
    exit_button.textContent = 'Close';
    exit_button.classList = 'close-button';

    populator(message);
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
