import {popup, put_into} from './common.js';

/**
 * @param {string} endpoint Where the request should go
 * @param {"GET" | "POST"} method The method to use
 * @param {string | undefined} body
 * @returns {Request}
 */
function make_request(endpoint, method = 'GET', body = undefined) {
    if (typeof body !== 'undefined')
        return new Request(`/simulation/${UUID}/${endpoint}`, {method, body});
    else
        return new Request(`/simulation/${UUID}/${endpoint}`, {method});
}

export const pipeline = {
    /** @type {HTMLTableElement} */
    table : document.getElementById('pipeline_table'),
    fetch : {
        /** @type {HTMLTableCellElement} */
        cell : document.getElementById('fetch_state'),
        /** @type {object} */
        state : null,
    },
    decode : {
        /** @type {HTMLTableCellElement} */
        cell : document.getElementById('decode_state'),
        /** @type {object} */
        state : null,
    },
    execute : {
        /** @type {HTMLTableCellElement} */
        cell : document.getElementById('execute_state'),
        /** @type {object} */
        state : null,
    },
    memory : {
        /** @type {HTMLTableCellElement} */
        cell : document.getElementById('memory_state'),
        /** @type {object} */
        state : null,
    },
    writeback : {
        /** @type {HTMLTableCellElement} */
        cell : document.getElementById('writeback_state'),
        /** @type {object} */
        state : null,
    },

    /** @type {() => void} */
    update : async function() {
        let r = await fetch(make_request('pipeline'));

        if (!r.ok)
            throw new Error(`Response: ${r.status}`);
        let values = await r.blob().then(b => b.text()).then(JSON.parse);

        this.fetch.state = values.fetch;
        this.fetch.cell.textContent = values.fetch.state;
        this.decode.state = values.decode;
        this.decode.cell.textContent = values.decode.state;
        this.execute.state = values.execute;
        this.execute.cell.textContent = values.execute.state;
        this.memory.state = values.memory;
        this.memory.cell.textContent = values.memory.state;
        this.writeback.state = values.writeback;
        this.writeback.cell.textContent = values.writeback?.job ?? 'idle';
    },

    /** @type {(stage: string) => void} */
    show_details : function(stage) {
        /** @type {string} */
        let stage_name = stage[0].toUpperCase() + stage.substring(1);
        if (this[stage].state !== null)
            popup(`Current State for Stage ${stage_name}`, container => {
                let list = document.createElement('ul');
                list.classList = 'configuration-display'
                container.appendChild(list);

                Object.entries(this[stage].state).forEach(put_into(list));
            });
        else
            popup(`Current State for Stage ${stage_name}`, container => {
                let list = document.createElement('ul');
                list.classList = 'configuration-display'
                container.appendChild(list);

                Object.entries({state : 'idle'}).forEach(put_into(list));
            });
    },
}

export const registers = {
    /** @type {HTMLTableElement} */
    table : document.getElementById('register_table'),
    /** @type {HTMLTableCellElement} */
    v0 : document.getElementById('reg_v0_val'),
    /** @type {HTMLTableCellElement} */
    v1 : document.getElementById('reg_v1_val'),
    /** @type {HTMLTableCellElement} */
    v2 : document.getElementById('reg_v2_val'),
    /** @type {HTMLTableCellElement} */
    v3 : document.getElementById('reg_v3_val'),
    /** @type {HTMLTableCellElement} */
    v4 : document.getElementById('reg_v4_val'),
    /** @type {HTMLTableCellElement} */
    v5 : document.getElementById('reg_v5_val'),
    /** @type {HTMLTableCellElement} */
    v6 : document.getElementById('reg_v6_val'),
    /** @type {HTMLTableCellElement} */
    v7 : document.getElementById('reg_v7_val'),
    /** @type {HTMLTableCellElement} */
    v8 : document.getElementById('reg_v8_val'),
    /** @type {HTMLTableCellElement} */
    v9 : document.getElementById('reg_v9_val'),
    /** @type {HTMLTableCellElement} */
    va : document.getElementById('reg_va_val'),
    /** @type {HTMLTableCellElement} */
    vb : document.getElementById('reg_vb_val'),
    /** @type {HTMLTableCellElement} */
    vc : document.getElementById('reg_vc_val'),
    /** @type {HTMLTableCellElement} */
    vd : document.getElementById('reg_vd_val'),
    /** @type {HTMLTableCellElement} */
    ve : document.getElementById('reg_ve_val'),
    /** @type {HTMLTableCellElement} */
    vf : document.getElementById('reg_vf_val'),
    /** @type {HTMLTableCellElement} */
    sp : document.getElementById('reg_sp_val'),
    /** @type {HTMLTableCellElement} */
    bp : document.getElementById('reg_bp_val'),
    /** @type {HTMLTableCellElement} */
    lp : document.getElementById('reg_lp_val'),
    /** @type {HTMLTableCellElement} */
    pc : document.getElementById('reg_pc_val'),
    /** @type {HTMLTableCellElement} */
    zf : document.getElementById('reg_zf_val'),
    /** @type {HTMLTableCellElement} */
    of : document.getElementById('reg_of_val'),
    /** @type {HTMLTableCellElement} */
    eps : document.getElementById('reg_eps_val'),
    /** @type {HTMLTableCellElement} */
    nan : document.getElementById('reg_nan_val'),
    /** @type {HTMLTableCellElement} */
    inf : document.getElementById('reg_inf_val'),

    update : async function() {
        let r = await fetch(make_request('registers'));

        if (!r.ok)
            throw new Error(`Response: ${r.status}`);
        let values = await r.blob().then(b => b.text()).then(JSON.parse);

        Object.entries(values).forEach(pair => {
            /** @type {string} */
            let reg = pair[0].toLowerCase();
            /** @type {{integer: number, float: number}} */
            let val = pair[1];

            const new_value = val.integer.toString(16).toUpperCase();

            if (new_value !== registers[reg].textContent)
                registers[reg].classList = 'updated';
            else
                registers[reg].classList = '';

            registers[reg].textContent = new_value;
            registers[reg].title = `int32: ${val.integer}\nfloat32: ${val.float}`;
        });
    }
};

export const watchlist = {
    /** @type {HTMLTableElement} */
    table : document.getElementById('watchlist'),
    /**
     * The list of entries in the table
     *
     * @type {{
     *    new_entry: (address: string, type: string, initial_value: string) => HTMLTableRowElement,
     *    [address: string]: {
     *        address: HTMLTableCellElement,
     *        type: HTMLTableCellElement,
     *        value: HTMLTableCellElement,
     *        row: HTMLTableRowElement
     *    }
     * }}
     */
    entries : {
        /**
         * @param {string} address
         * @param {string} type
         * @param {string} initial_value
         * @returns {HTMLTableRowElement}
         */
        new_entry : function(address, type, initial_value) {
            let new_entry = {
                address : document.createElement('th'),
                type : document.createElement('td'),
                value : document.createElement('td'),
                row : document.createElement('tr'),
            };

            new_entry.address.textContent = Number.parseInt(address).toString(16).toUpperCase().padStart(8, '0');
            new_entry.address.classList = 'monospace';
            new_entry.type.textContent = type;
            new_entry.value.textContent = initial_value;
            new_entry.value.classList = 'monospace';

            let del_cel = document.createElement('td');
            let delete_button = document.createElement('button');
            del_cel.appendChild(delete_button);
            delete_button.textContent = 'Delete';
            delete_button.onclick = () => watchlist.remove_entry(address);

            new_entry.row.append(document.createElement('th'), new_entry.address, new_entry.type, new_entry.value,
                                 del_cel);

            this[address] = new_entry;

            return new_entry.row;
        }
    },

    /** @type {HTMLInputElement} */
    address : document.getElementById('watchlist_address'),
    /** @type {HTMLOptionElement} */
    type : document.getElementById('watchlist_type'),

    get_entries : async function() {
        let response = await fetch(make_request('watchlist', 'POST', JSON.stringify({})));
        if (!response.ok)
            throw new Error(`Response: ${response.status}`);

        let values = await response.blob().then(b => b.text()).then(JSON.parse);

        Object.entries(values).forEach(
            pair => this.table.appendChild(this.entries.new_entry(pair[0], pair[1][0], pair[1][1])));
    },

    update : async function() {
        let response = await fetch(make_request('watchlist'));

        if (!response.ok)
            throw new Error(`Response: ${response.status}`);

        let values = await response.blob().then(b => b.text()).then(JSON.parse);
        Object.entries(values).forEach(pair => {
            const entry = this.entries[pair[0]];

            if (entry.value.textContent !== pair[1])
                entry.value.classList = 'updated monospace';
            else
                entry.classList = 'monospace';

            entry.value.textContent = pair[1];
        });
    },

    /** @type {() => void} */
    add_new_entry : async function() {
        let entry = {};
        entry[this.address.value] = this.type.value;

        let response = await fetch(make_request('watchlist', 'POST', JSON.stringify(entry)));
        if (!response.ok)
            throw new Error(`Response: ${response.status}`);

        let values = await response.blob().then(b => b.text()).then(JSON.parse);
        let a = this.address.value;

        if (typeof this.entries[a] === 'undefined') {
            this.table.appendChild(this.entries.new_entry(a, values[a][0], values[a][1]));
        } else {
            this.entries[a].type.textContent = values[a][0];
        }

        Object.entries(values).forEach(pair => this.entries[pair[0]].value.textContent = pair[1][1]);
    },

    /** @type {(address: number?) => void} */
    remove_entry : async function(address) {
        if (typeof this.entries[address] !== 'undefined') {
            this.table.removeChild(this.entries[address].row);
            delete this.entries[address];
        }

        let entry = {};
        entry[address] = null;

        let response = await fetch(make_request('watchlist', 'POST', JSON.stringify(entry)));

        if (!response.ok)
            throw new Error('Failed request!');

        let values = await response.blob().then(b => b.text()).then(JSON.parse);

        Object.entries(values).forEach(pair => this.entries[pair[0]].value.textContent = pair[1][1])
    },
}

const MEMORY_COLUMNS = 16;
const MEMORY_CELLS = 16384;
const VIEW_PAGES = PAGE_SIZE / MEMORY_CELLS * PAGE_COUNT;
export const memory = {
    /** @type {HTMLTableElement} */
    table : document.getElementById('memory_table'),
    /** @type {HTMLSelectElement} */
    selector : document.getElementById('memoryview_selector'),
    /** @type {number} */
    page_id : 0,
    /** @type {HTMLButtonElement} */
    prev_button : document.getElementById('memoryview_prev'),
    /** @type {HTMLParagraphElement} */
    label : document.getElementById('memview_pageid'),
    /** @type {HTMLButtonElement} */
    next_button : document.getElementById('memoryview_next'),

    /** @type {string} */
    last_hash : null,

    /** @type {() => void} */
    next_page : function() {
        this.page_id += 1;
        if (this.page_id >= VIEW_PAGES)
            this.page_id = VIEW_PAGES - 1;
        else {
            this.last_hash = null;
            this.update();
        }
    },
    /** @type {() => void} */
    prev_page : function() {
        this.page_id -= 1;
        if (this.page_id < 0)
            this.page_id = 0;
        else {
            this.last_hash = null;
            this.update();
        }
    },

    /** @type {() => void} */
    update : async function() {
        let disasm = this.selector.value === 'disassembly';
        let params = new URLSearchParams({hash : this.last_hash})

        let r = await fetch(
            make_request(disasm ? `page/${memory.page_id}?${params}&disasm` : `page/${memory.page_id}?${params}`));

        if (!r.ok)
            throw new Error(`Response: ${r.status}`);

        /** @type {{data: number[], hash: number}|null} */
        let values = await r.blob().then(b => b.text()).then(JSON.parse);

        let new_page = this.last_hash === null;

        if (values !== null)
            this.last_hash = values.hash;

        for (let i = 0; i < MEMORY_CELLS / MEMORY_COLUMNS; ++i)
            memory.table.rows[i + 1].cells[0].textContent =
                (memory.page_id * MEMORY_CELLS + i * MEMORY_COLUMNS).toString(16).toUpperCase().padStart(8, '0');

        const cell_count = disasm ? MEMORY_CELLS / 4 : MEMORY_CELLS;
        const col_count = disasm ? MEMORY_COLUMNS / 4 : MEMORY_COLUMNS;
        const i_inc = disasm ? 4 : 1;

        for (let i = 0; i < cell_count; ++i) {
            let row = ((i / col_count) | 0) + 1;
            let col = i % col_count + 1;
            let cell = memory.table.rows[row].cells[col];
            let address = (memory.page_id * MEMORY_CELLS + i * i_inc);
            if (values !== null) {
                let new_value = values.data[i].toString(16).toUpperCase().padStart(2, '0');
                if (cell.textContent !== new_value && !new_page)
                    cell.classList = 'updated';
                else
                    cell.classList = '';
                cell.textContent = new_value;
            } else if (new_page) {
                cell.textContent = disasm ? '' : '00';
                cell.classList = '';
            }
            cell.title = `Address: ${address.toString(16).toUpperCase().padStart(8, '0')}\nDecimal Address: ${address}`;
        }

        memory.label.textContent = `${memory.page_id + 1} / ${VIEW_PAGES}`
    },

    /**
     * Populates a table with cells
     */
    populate : function() {
        let is_disasm = this.selector.value === 'disassembly';

        while (this.table.firstChild !== null)
            this.table.removeChild(this.table.lastChild);

        /** @type {HTMLTableRowElement} */
        let header_row = document.createElement('tr');
        header_row.classList = 'header-row';
        this.table.appendChild(header_row);
        header_row.appendChild(document.createElement('th'));

        for (let i = 0; i < MEMORY_COLUMNS; ++i) {
            let cell = document.createElement('th');
            cell.textContent = i.toString(16).toUpperCase();
            header_row.appendChild(cell);
        }

        for (let i = 0; i < MEMORY_CELLS / MEMORY_COLUMNS; ++i) {
            let row = document.createElement('tr');
            this.table.appendChild(row);
            let address = document.createElement('th');
            row.appendChild(address);

            if (is_disasm)
                for (let j = 0; j < MEMORY_COLUMNS && i * MEMORY_COLUMNS + j < MEMORY_CELLS; j += 4) {
                    let cell = document.createElement('td');
                    cell.colSpan = 4;
                    row.appendChild(cell);
                }
            else
                for (let j = 0; j < MEMORY_COLUMNS && i * MEMORY_COLUMNS + j < MEMORY_CELLS; ++j) {
                    let cell = document.createElement('td');
                    row.appendChild(cell);
                }
        }
    },

    /** @type {() => void} */
    switch_mode : function() {
        this.populate();
        this.last_hash = null;
        this.update();
    }
};

export const cache = {
    /** @type {HTMLSelectElement} */
    selector : document.getElementById('cacheview_selector'),
    /** @type {HTMLDivElement} */
    content : document.getElementById('cacheview_content'),
    /**
     * @type {{[name: string]: {mode: "associative", set_bits: number, offset_bits: number, ways: number} | {mode:
     *     "disabled"}}}
     */
    configurations : {},
    /** @type {[name: string]: HTMLTableElement|HTMLParagraphElement} */
    tables : {},
    /** @type {() => void} */
    initialize : async function() {
        let r = await fetch(make_request(`cache/names`));

        if (!r.ok)
            throw new Error(`Response: ${r.status}`);

        /**
         * @type {{[name: string]: {mode: "associative", set_bits: number, offset_bits: number, ways: number} | {mode:
         *     "disabled"}}}
         */
        let values = await r.blob().then(b => b.text()).then(JSON.parse);

        /**
         *
         * @param {string} name
         * @param {{mode: "associative", set_bits: number, offset_bits: number, ways: number} | {mode:
         *     "disabled"}} config
         */
        let make_table = (name, config) => {
            if (config.mode === 'disabled') {
                let p = document.createElement('p');
                p.textContent = 'No data';
                this.tables[name] = p;
            } else {
                let table = document.createElement('table');
                let headers = document.createElement('tr');
                ['Set', 'Way', 'Base Address', 'Dirty']
                    .map(n => {
                        let h = document.createElement('th');
                        h.textContent = n;
                        return h;
                    })
                    .forEach(c => headers.appendChild(c));
                let wide_header = document.createElement('th');
                wide_header.textContent = 'Values';
                wide_header.colSpan = Math.pow(2, config.offset_bits);
                headers.appendChild(wide_header);

                let rows = [ headers ];

                for (let set = 0; set < Math.pow(2, config.set_bits); ++set) {
                    let first_row = document.createElement('tr');
                    let first_row_cells = [
                        document.createElement('th'),
                        document.createElement('th'),
                        document.createElement('th'),
                        document.createElement('th'),
                    ];
                    first_row_cells[0].textContent = `${set}`;
                    first_row_cells[0].rowSpan = config.ways;
                    first_row_cells[1].textContent = "0";

                    for (let offset = 0; offset < Math.pow(2, config.offset_bits); ++offset)
                        first_row_cells.push(document.createElement('td'));

                    first_row_cells.forEach(c => c.classList = 'monospace');

                    first_row_cells.forEach(c => first_row.appendChild(c));
                    rows.push(first_row);

                    for (let way = 1; way < config.ways; ++way) {
                        let row = document.createElement('tr');
                        let cells = [
                            document.createElement('th'),
                            document.createElement('th'),
                            document.createElement('th'),
                        ];
                        cells[0].textContent = `${way}`;

                        for (let offset = 0; offset < Math.pow(2, config.offset_bits); ++offset)
                            cells.push(document.createElement('td'));

                        cells.forEach(c => c.classList = 'monospace');

                        cells.forEach(c => row.appendChild(c));
                        rows.push(row);
                    }
                }

                rows.forEach(r => table.appendChild(r));
                this.tables[name] = table;
            }
        };

        Object.entries(values).forEach(v => {
            let opt = document.createElement('option');
            opt.textContent = v[0][0].toUpperCase() + v[0].substring(1);
            opt.value = v[0];

            this.selector.appendChild(opt);

            this.configurations[v[0]] = v[1];
            make_table(v[0], v[1]);
        });

        await this.update(true);
    },
    /** @type {(swap: boolean) => void} */
    update : async function(swap) {
        let cache = this.selector.value;

        let r = await fetch(make_request(`cache/read/${cache}`));

        if (!r.ok)
            throw new Error(`Response: ${r.status}`);

        /** @type {({base_address: number, dirty: boolean, data: number[]}|null)[]} */
        let values = await r.blob().then(b => b.text().then(JSON.parse));
        /** @type {HTMLTableElement} */
        let table = this.tables[cache];
        let config = this.configurations[cache];

        if (swap) {
            this.content.childNodes.forEach(c => c.remove());
            this.content.append(table);
        }

        if (config.mode.toLowerCase() === 'associative')
            values.forEach((r, i) => {
                let tr = table.rows[i + 1];
                const col_offset = (i % config.ways === 0) ? 1 : 0;

                let ba = tr.cells[col_offset + 1];
                let db = tr.cells[col_offset + 2];

                if (r === null) {
                    ba.textContent = '';
                    db.textContent = '';
                } else {
                    ba.textContent = r.base_address.toString(16).toUpperCase().padStart(8, '0');
                    db.textContent = r.dirty;
                }
                if (r === null)
                    for (let i = 3 + col_offset; i < tr.cells.length; ++i)
                        tr.cells[i].textContent = '';
                else
                    for (let i = 0; i < r.data.length; ++i)
                        tr.cells[i + col_offset + 3].textContent =
                            r.data[i].toString(16).toUpperCase().padStart(2, '0');
            });
    }
};

memory.populate();

pipeline.update();
registers.update();
watchlist.get_entries();
memory.update();
cache.initialize();

export async function show_config() {
    let r = await fetch(make_request(`configuration`));

    if (!r.ok)
        throw new Error(`Response: ${r.status}`);

    /** @type {Promise<object>} */
    let values = r.blob().then(b => b.text()).then(JSON.parse);

    popup('Configuration', async container => {
        let list = document.createElement('ul');
        list.classList = 'configuration-display'
        container.appendChild(list);

        container.appendChild(document.createElement('hr'));

        Object.entries(await values).forEach(put_into(list));
    });
}

const clock_button = document.getElementById("clock_button");
const step_button = document.getElementById("step_button");
const run_button = document.getElementById("run_button");

function disable_header_buttons() {
    clock_button.onclick = null;
    step_button.onclick = null;
    run_button.onclick = null;
}

function enable_header_buttons() {
    clock_button.onclick = clock;
    step_button.onclick = step;
    run_button.onclick = run;
}

export async function clock() {
    disable_header_buttons();

    let r = await fetch(make_request('clock', 'POST'));

    if (!r.ok)
        throw new Error(`Response: ${r.status} ${r.statusText}`);

    console.log(await r.blob().then(b => b.text()));

    let tasks = [ registers.update(), watchlist.update(), memory.update(), pipeline.update(), cache.update(false) ];

    await Promise.allSettled(tasks);

    enable_header_buttons();
}

export async function step() {
    disable_header_buttons();

    let r = await fetch(make_request('step', 'POST'));

    if (!r.ok)
        throw new Error(`Response: ${r.status} ${r.statusText}`);

    console.log(await r.blob().then(b => b.text()));

    let tasks = [ registers.update(), watchlist.update(), memory.update(), pipeline.update(), cache.update(false) ];

    await Promise.allSettled(tasks);

    enable_header_buttons();
}

export async function run() {
    disable_header_buttons();

    let r = await fetch(make_request('run', 'POST'));

    if (!r.ok)
        throw new Error(`Response: ${r.status} ${r.statusText}`);

    console.log(await r.blob().then(b => b.text()));

    let tasks = [ registers.update(), watchlist.update(), memory.update(), pipeline.update(), cache.update(false) ];

    await Promise.allSettled(tasks);

    enable_header_buttons();
}
