# flysoar

CLI for searching flights via [FlySoar.ai](https://flysoar.ai)'s public API. No accounts, API keys, or authentication required.

## Features

- One-way, round-trip, open-jaw, and multi-city search
- Accept search params as CLI flags or raw JSON via `--input`
- Output as JSON (default), CSV, or colored table
- Raw mode (`--raw`) outputs full unmodified API offers
- Sort by price, departure time, or duration
- Filter to nonstop flights only
- Limit number of offers returned
- Save results to file with `--save`
- Quiet mode (`--quiet`) for clean piped output
- Progress bar during search (hidden in quiet mode)
- Self-update (`flysoar update`)
- Installation info (`flysoar info`)
- Self-uninstall (`flysoar uninstall`)

## Installation

### Prebuilt binaries (macOS and Linux)

Install the latest GitHub Release binary:

```bash
curl -fsSL https://raw.githubusercontent.com/Gahnxd/flysoar-cli/main/install.sh | sh
```

To install a specific version or directory:

```bash
curl -fsSL https://raw.githubusercontent.com/Gahnxd/flysoar-cli/main/install.sh | \
  FLYSOAR_VERSION=v0.1.0 FLYSOAR_INSTALL_DIR="$HOME/.local/bin" sh
```

The installer verifies the release checksum before installing. Windows users can download the ZIP archive from the [GitHub Releases page](https://github.com/Gahnxd/flysoar-cli/releases).

### From source

```bash
cargo install --path .
```

This installs the `flysoar` binary to `~/.cargo/bin/`. Make sure `~/.cargo/bin` is on your PATH.

```bash
flysoar search -o SFO -d JFK -D 2026-07-15
```

Build the binary without installing:

```bash
cargo build --release
# Binary at target/release/flysoar (not added to PATH)
```

```bash
./target/release/flysoar search -o SFO -d JFK -D 2026-07-15
```

## Uninstall

```bash
flysoar uninstall
```

This removes whichever binary is currently running (works regardless of whether you installed via `install.sh` or `cargo install`).

If you installed via cargo, you can alternatively run:

```bash
cargo uninstall flysoar
```

## Usage

```
CLI for searching flights via FlySoar.ai's public SSE endpoint.
No accounts, API keys, or authentication required.

Supports one-way, round-trip, open-jaw, and multi-city searches.
Output formats: JSON (default), CSV, or table.

Usage: flysoar <COMMAND>

Commands:
  search     Search for flights
  update     Self-update: reinstall flysoar from source
  info       Show installation info
  uninstall  Uninstall the flysoar binary
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Search

```
flysoar search --help
```

```
Search for flight offers.

Examples:
  flysoar search -o SFO -d JFK -D 2026-07-15
  flysoar search -o SFO -d JFK -D 2026-07-15 -r 2026-07-22 -c business
  flysoar search --slices SFO,JFK,2026-07-15 LHR,SFO,2026-07-22 -c economy
  flysoar search --input '{"origin":"SFO","destination":"JFK","date":"2026-07-15"}'

Options:
  -o, --origin <ORIGIN>            Departure IATA airport or city code
  -d, --destination <DESTINATION>  Arrival IATA airport or city code
  -D, --date <DATE>                Outbound departure date (YYYY-MM-DD)
  -r, --return-date <RETURN_DATE>  Return date for round-trips (YYYY-MM-DD)
      --slices <SLICES>...         Multi-city slices: "ORIGIN,DEST,DATE" (repeatable)
      --input <INPUT>              Accept a JSON search body directly
  -c, --cabin <CABIN>              Cabin class [default: economy]
  -p, --passengers <PASSENGERS>    Number of passengers [default: 1]
  -O, --output <OUTPUT>            Output format: json, csv, table [default: json]
  -n, --max-offers <MAX_OFFERS>    Max offers to return (0 = unlimited) [default: 0]
  -t, --timeout <TIMEOUT>          Request timeout in seconds [default: 90]
  -s, --sort <SORT>                Sort by: price, depart_time, duration [default: price]
      --nonstop-only               Filter to nonstop flights only
  -q, --quiet                      Suppress all progress output
      --raw                        Output raw unmodified API offers (full schema)
      --save <SAVE>                Save results to a file (JSON format)
  -h, --help                       Print help
```

### Examples

**One-way economy:**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15
```

**Round-trip business:**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15 -r 2026-07-22 -c business
```

**Multi-city (open-jaw):**
```bash
flysoar search --slices SFO,JFK,2026-07-15 LHR,SFO,2026-07-22
```

**Table output, nonstop only, sorted by price:**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15 -O table --nonstop-only -s price
```

**CSV export, max 20 offers:**
```bash
flysoar search -o NYC -d LON -D 2026-08-01 -O csv -n 20 > flights.csv
```

**JSON input (agent-friendly):**
```bash
flysoar search --input '{"origin":"SFO","destination":"JFK","date":"2026-07-15","cabin":"business"}'
```

**Raw output (full API schema):**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15 --raw -q
```

**Save to file:**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15 --save results.json
```

**Quiet mode (clean stdout for piping):**
```bash
flysoar search -o SFO -d JFK -D 2026-07-15 -q | jq '.offers[0].price'
```

### update

Reinstall flysoar from the current source directory:
```bash
flysoar update
flysoar update --path /path/to/flysoar_cli
```

### info

Show installation status and binary location:
```bash
flysoar info
```

## Parameters

| Parameter | Format | Required | Default | Notes |
|-----------|--------|----------|---------|-------|
| `origin` | IATA code | flat mode | — | Airport or city code (e.g. `SFO`, `NYC`, `LON`) |
| `destination` | IATA code | flat mode | — | Same as origin |
| `date` | `YYYY-MM-DD` | flat mode | — | Must be a future date |
| `return_date` | `YYYY-MM-DD` | no | — | Omit for one-way |
| `slices` | `ORIGIN,DEST,DATE` | multi-city | — | Repeatable for multiple legs |
| `cabin` | string | no | `economy` | `economy`, `premium_economy`, `business`, `first` |
| `passengers` | integer | no | `1` | Values ≤ 0 treated as 1 |

## Output Formats

### JSON (default)

Pretty-printed JSON with query metadata and offer summaries:

```json
{
  "query": { "origin": "SFO", "destination": "JFK", ... },
  "offers": [ { "price": 368.40, "airline": "American Airlines", ... } ],
  "count": 42
}
```

### Table

Colored UTF-8 table with price, airline, flight number, route, times, stops, duration, and cabin.

### CSV

Flat CSV with columns: price, currency, stops, airline, flight_numbers, depart_time, arrive_time, duration, cabin_class, route, emissions_kg.

## How It Works

The CLI sends a `POST` request to `https://flysoar.ai/api/search/stream` and parses the Server-Sent Events (SSE) stream, collecting `event: offer` payloads until `event: done`. No API key or authentication is needed — the endpoint is fully public.