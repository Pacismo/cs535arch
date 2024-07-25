# SEIS Simulation Server

- [SEIS Simulation Server](#seis-simulation-server)
  - [Notes](#notes)
  - [Endpoints](#endpoints)
    - [GET `/` and `/<PATH>`](#get--and-path)
    - [POST `/simulation`](#post-simulation)
    - [GET `/simulation/<UUID>`](#get-simulationuuid)
    - [GET `/simulation/<UUID>/page/<PAGE_ID>?<LAST_HASH>`](#get-simulationuuidpagepage_idlast_hash)
    - [GET `/simulation/<UUID>/configuration`](#get-simulationuuidconfiguration)
    - [GET `/simulation/<UUID>/memory/<ADDRESS>/<TYPE>`](#get-simulationuuidmemoryaddresstype)
    - [GET `/simulation/<UUID>/watchlist`](#get-simulationuuidwatchlist)
    - [POST `/simulation/<UUID>/watchlist`](#post-simulationuuidwatchlist)
    - [GET `/simulation/<UUID>/registers`](#get-simulationuuidregisters)
    - [GET `/simulation/<UUID>/pipeline`](#get-simulationuuidpipeline)

## Notes

This application uses Rocket to host a handful of web pages and provision an API to create and access the data pertaining to an instance of the simulation.

The `web` dir contains webpage content useful for generating the app's web pages. It is copied to the installation directory when building the software.

The page for each simulation is generated on-the-fly and provides the UUID of the simulation as a global constant to the client. Changing the HTML file at `src/api/app/application.html` will require rebuilding the codebase (be sure that the format strings at the top of the file have no whitespaces within the curly braces).

Invalid UUIDs yield a 404 error.

## Endpoints

### GET `/` and `/<PATH>`

When not matched to an explicit route (such as `simulation`), will fetch the appropriate file from the `web` directory.

If the path arrives at a directory, returns the `index.html` file at that directory.

### POST `/simulation`

Initializes the simulation with the provided JSON data. Returns the UUID of the newly-created session if successful.

It will return an error describing why the request failed -- including detailed assembly errors.

JSON Schema:

```ts
{
  miss_penalty: number, // integer
  volatile_penalty: number, // integer
  pipelining: boolean,
  writethrough: boolean,
  // requires at least `instruction` and `data` caches defined,
  // as they are the only available caches as of now
  cache: {
    [name: string]: {
      mode: "Associative" | "Disabled", // not case-sensitive
      set_bits?: number, // integer; required for "associative" mode
      offset_bits?: number, // integer; required for "associative" mode
      ways?: number // integer; required for "associative" mode
    }
  },
  // requires at least one file. key is the name, value is the contents
  files: {
    [filename: string]: string
  }
}
```

### GET `/simulation/<UUID>`

The dashboard for the simulation with ID `UUID`.

### GET `/simulation/<UUID>/page/<PAGE_ID>?<LAST_HASH>`

Reads the page contents for the specified simulation; if `LAST_HASH` is not null, the page is only transmitted if the hashes do not match. Additionally, returns `null` if the page is not allocated.

In this context, a page is 16 KiB, or a quarter of the size of a "page" as allocated by a simulation (64KiB).

Response body schema:

```ts
{
  hash: string, // JS has rounding errors
  data: number[], // These are the 16384 bytes for the page
}
```

### GET `/simulation/<UUID>/configuration`

Gets a JSON object representing the current working configuration for the specified simulation.

This does *not* include the names or contents of the files passed to the assembler.

### GET `/simulation/<UUID>/memory/<ADDRESS>/<TYPE>`

Reads data of type `TYPE` from memory address `ADDRESS` for a given simulation instance `UUID`.

### GET `/simulation/<UUID>/watchlist`

Reads the memory addresses listed in the instance's watchlist.

Returns a JSON object of key-value pairs, of addresses mapped to the string representation of the value at that address in the type specified in the watchlist.

Response body schema:

```ts
{
  [address: string]: string
}
```

### POST `/simulation/<UUID>/watchlist`

Modifies the watchlist. New entries will . A type of `null` removes the entry.

Request body schema:

```ts
{
  [address: string]: "byte" | "short" | "word" | "float" | null
}
```

Returns a JSON object representing the entries of the watchlist, their types, and their current values.

Response body schema:

```ts
{
  [address: string]: ["byte" | "short" | "word" | "float", string]
}
```

### GET `/simulation/<UUID>/registers`

Reads the registers as a JSON object, representing those registers in key-value pairs (variable registers are stored under `v` as an array).

### GET `/simulation/<UUID>/pipeline`

Reads the state of the pipeline into a JSON object. The schema varies wildly depending on what the state of each stage is.
