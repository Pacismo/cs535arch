import {popup} from "./common.js"

export async function submit_form() {
    const miss_penalty = Number.parseInt(document.getElementById("conf_mem_misspenalty").value);
    const volatile_penalty = Number.parseInt(document.getElementById("conf_mem_volatilepenalty").value);
    const pipelining = document.getElementById("conf_mem_pipeliningenabled").value === "on";
    const writethrough = document.getElementById("conf_mem_writethrough").value === "on";
    const data_set_bits = Number.parseInt(document.getElementById("conf_cache_data_setbits").value);
    const data_offset_bits = Number.parseInt(document.getElementById("conf_cache_data_offsetbits").value);
    const data_ways = Number.parseInt(document.getElementById("conf_cache_data_ways").value);
    const inst_set_bits = Number.parseInt(document.getElementById("conf_cache_instruction_setbits").value);
    const inst_offset_bits = Number.parseInt(document.getElementById("conf_cache_instruction_offsetbits").value);
    const inst_ways = Number.parseInt(document.getElementById("conf_cache_instruction_ways").value);
    /** @type {HTMLInputElement} */
    const asm = document.getElementById("asm_file");

    let files = {};

    for (let i = 0; i < asm.files.length; ++i)
        files[asm.files[i].name] = await asm.files[i].text();

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
