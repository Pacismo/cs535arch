# SEIS Simulation Server

- [SEIS Simulation Server](#seis-simulation-server)
  - [Notes](#notes)
  - [Endpoints](#endpoints)
    - [GET `/` and `/<PATH>`](#get--and-path)
    - [POST `/simulation`](#post-simulation)
    - [GET `/simulation/<UUID>`](#get-simulationuuid)
    - [GET `/simulation/<UUID>/memory/<ADDRESS>/<TYPE>`](#get-simulationuuidmemoryaddresstype)
    - [GET `/simulation/<UUID>/watchlist`](#get-simulationuuidwatchlist)
    - [GET `/simulation/<UUID>/registers`](#get-simulationuuidregisters)

## Notes

This application uses Rocket to host a handful of web pages and provision an API to create and access the data pertaining to an instance of the simulation.

The `web` dir contains webpage content useful for generating the app's web pages. It is copied to the installation directory when building the software.

Invalid UUIDs yield a 404 error.

## Endpoints

### GET `/` and `/<PATH>`

When not matched to an explicit route (such as `simulation`), will fetch the appropriate file from the `web` directory.

If the path arrives at a directory, returns the `index.html` file at that directory.

### POST `/simulation`

Initializes the simulation with the provided form data. Sends back a webpage that redirects to the dashboard for that new simulation.

### GET `/simulation/<UUID>`

The dashboard for the simulation with ID `UUID`.

### GET `/simulation/<UUID>/memory/<ADDRESS>/<TYPE>`

Reads data of type `TYPE` from memory address `ADDRESS` for a given simulation instance `UUID`.

### GET `/simulation/<UUID>/watchlist`

Reads the memory addresses listed in the instance's watchlist.

Returns a JSON object of key-value pairs, of addresses mapped to the string representation of the value at that address in the type specified in the watchlist.

### GET `/simulation/<UUID>/registers`

Reads the registers as a JSON object, representing those registers in key-value pairs (variable registers are stored under `v` as an array).
