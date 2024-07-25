async function submit_form() {
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

    let asm_file = asm.files[0].name;
    let asm_data = await asm.files[0].text();

    let data = data_ways !== 0
                   ? {set_bits : data_set_bits, offset_bits : data_offset_bits, ways : data_ways, mode : "associative"}
                   : {mode : "disabled"};
    let instruction =
        inst_ways !== 0
            ? {set_bits : inst_set_bits, offset_bits : inst_offset_bits, ways : inst_ways, mode : "associative"}
            : {mode : "disabled"};

    let cache = {data, instruction};

    let body = {miss_penalty : miss_penalty, volatile_penalty, pipelining, writethrough, cache, asm_data, asm_file};

    let response = await fetch(new Request("/simulation", {method : "POST", body : JSON.stringify(body)}));

    if (!response.ok)
        throw `HTTP status: ${response.status} ${await response.blob().then(r => r.text())}`;

    let uuid = await response.blob().then(r => r.text());

    window.location.assign(`/simulation/${uuid}`);
}
