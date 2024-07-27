import {popup} from "./common.js"

const conf = {
    /** @type {HTMLInputElement} */
    miss_penalty : document.getElementById("conf_mem_misspenalty"),
    /** @type {HTMLInputElement} */
    volatile_penalty : document.getElementById("conf_mem_volatilepenalty"),
    /** @type {HTMLInputElement} */
    pipelining : document.getElementById("conf_mem_pipeliningenabled"),
    /** @type {HTMLInputElement} */
    writethrough : document.getElementById("conf_mem_writethrough"),
    /** @type {HTMLInputElement} */
    cache_data_setbits : document.getElementById("conf_cache_data_setbits"),
    /** @type {HTMLInputElement} */
    cache_data_offsetbits : document.getElementById("conf_cache_data_offsetbits"),
    /** @type {HTMLInputElement} */
    cache_data_ways : document.getElementById("conf_cache_data_ways"),
    /** @type {HTMLInputElement} */
    cache_instruction_setbits : document.getElementById("conf_cache_instruction_setbits"),
    /** @type {HTMLInputElement} */
    cache_instruction_offsetbits : document.getElementById("conf_cache_instruction_offsetbits"),
    /** @type {HTMLInputElement} */
    cache_instruction_ways : document.getElementById("conf_cache_instruction_ways"),
    /** @type {HTMLInputElement} */
    asm : document.getElementById("asm_file"),
};

export async function submit_form() {
    const miss_penalty = Number.parseInt(conf.miss_penalty.value);
    const volatile_penalty = Number.parseInt(conf.volatile_penalty.value);
    const pipelining = conf.pipelining.checked;
    const writethrough = conf.writethrough.checked;
    const data_set_bits = Number.parseInt(conf.cache_data_setbits.value);
    const data_offset_bits = Number.parseInt(conf.cache_data_offsetbits.value);
    const data_ways = Number.parseInt(conf.cache_data_ways.value);
    const inst_set_bits = Number.parseInt(conf.cache_instruction_setbits.value);
    const inst_offset_bits = Number.parseInt(conf.cache_instruction_offsetbits.value);
    const inst_ways = Number.parseInt(conf.cache_instruction_ways.value);

    let files = {};

    for (let i = 0; i < conf.asm.files.length; ++i)
        files[conf.asm.files[i].name] = await conf.asm.files[i].text();

    let data = data_ways !== 0
                   ? {set_bits : data_set_bits, offset_bits : data_offset_bits, ways : data_ways, mode : "associative"}
                   : {mode : "disabled"};
    let instruction =
        inst_ways !== 0
            ? {set_bits : inst_set_bits, offset_bits : inst_offset_bits, ways : inst_ways, mode : "associative"}
            : {mode : "disabled"};

    let cache = {data, instruction};

    let body = {miss_penalty, volatile_penalty, pipelining, writethrough, cache, files};

    let response = await fetch(new Request("/simulation", {method : "POST", body : JSON.stringify(body)}));

    if (!response.ok) {
        popup(`Error ${response.status}`, async container => {
            container.classList = 'error-message';

            (await response.blob().then(r => r.text())).split(/\n|\r\n/).forEach(l => {
                let p = document.createElement('code');
                p.classList = 'monospace';
                p.style = "margin-top: 0; margin-bottom: 0; display: block; white-space: pre;";
                container.appendChild(p);
                p.textContent = l;
            });
        });

        return;
    }

    let uuid = await response.blob().then(r => r.text());

    window.location.assign(`/simulation/${uuid}`);
}
