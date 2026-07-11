---
name: flysoar-flight-search
description: Search and compare live flight offers with the flysoar CLI. Use when a user asks for current flight options, fares, airlines, schedules, nonstop choices, or multi-city/open-jaw itineraries; install the CLI when needed, run a search, and report time-sensitive results without booking.
---

# FlySoar Flight Search

Use the `flysoar` CLI by default: it constructs the request, parses the SSE stream, and formats or filters offers. Use `curl` only as a fallback when the CLI cannot be installed or run, or when diagnosing a CLI/API discrepancy. Return to the CLI for normal searches.

## Install and verify

Check whether `flysoar` is available:

```bash
flysoar --version
```

When working from this repository and the command is unavailable, install it with Cargo:

```bash
cargo install --path .
flysoar --version
```

Ensure `~/.cargo/bin` is on `PATH` after installation. Build and invoke `./target/release/flysoar` when a global install is unsuitable.

## Gather the search details

Obtain or infer the origin, destination, future departure date, return date when applicable, cabin, and passenger count. Use three-letter IATA airport or city codes. Valid cabins are `economy`, `premium_economy`, `business`, and `first`; default to economy and one passenger.

Use city codes such as `NYC` or `LON` for city-wide results. Avoid a city code and one of its own airports as a route (for example, `NYC` to `JFK`), because it can return no offers.

## Run searches

Use JSON output with quiet mode for reliable programmatic handling. The default sort is price.

```bash
# One-way
flysoar search -o SFO -d JFK -D YYYY-MM-DD -q

# Round trip
flysoar search -o SFO -d JFK -D YYYY-MM-DD -r YYYY-MM-DD -c business -p 1 -q

# Open-jaw or multi-city; repeat ORIGIN,DESTINATION,DATE slices
flysoar search --slices SFO,JFK,YYYY-MM-DD LHR,SFO,YYYY-MM-DD -c economy -q
```

Apply common user constraints through the CLI rather than post-processing raw SSE output:

```bash
# Cheapest nonstop offers in a readable table
flysoar search -o SFO -d JFK -D YYYY-MM-DD \
  --nonstop-only --sort price --max-offers 10 --output table

# CSV for a spreadsheet or JSON saved for later inspection
flysoar search -o NYC -d LON -D YYYY-MM-DD --output csv --max-offers 20
flysoar search -o SFO -d JFK -D YYYY-MM-DD --save results.json -q
```

Use `--input` when the user supplies a complete request body. Use `--raw -q` only when the response details below are needed; ordinary JSON output contains normalized offer summaries.

```bash
flysoar search --input '{"origin":"SFO","destination":"JFK","date":"YYYY-MM-DD","cabin":"business","passengers":1}' -q
flysoar search -o SFO -d JFK -D YYYY-MM-DD --raw -q
```

| Option | Use |
| --- | --- |
| `-r`, `--return-date` | Add a return date for a round trip. |
| `--slices` | Supply one or more `ORIGIN,DESTINATION,DATE` legs for multi-city or open-jaw travel. |
| `-p`, `--passengers` | Set passenger count; default is `1`. |
| `-O`, `--output` | Choose `json` (default), `csv`, or `table`. |
| `-n`, `--max-offers` | Limit returned offers; `0` means unlimited. |
| `-s`, `--sort` | Sort by `price`, `depart_time`, or `duration`. |
| `--nonstop-only` | Retain only offers whose every slice has one segment. |
| `-q`, `--quiet` | Suppress progress output; use for JSON or CSV pipelines. |
| `--raw` | Return unmodified API offers. |
| `--save` | Write normalized JSON results to a file. |
| `-t`, `--timeout` | Set the request timeout in seconds; default is `90`. |

## Report results responsibly

Apply the user's constraints before comparing offers. For each shortlisted offer, report total price and currency, airline and flight number, route, local departure and arrival times, stops, duration, and cabin. Include material caveats present in the result, including codeshares, fare conditions, baggage, or expiry.

State the sort and filters used. Treat results as time-sensitive estimates: prices and availability can change, and the CLI does not book travel. If no offer matches, say so instead of broadening the search without permission.

## Curl fallback and API behavior

The CLI uses `POST https://flysoar.ai/api/search/stream`, which returns Server-Sent Events (SSE). Use the fallback only after recording why the CLI was unavailable or unsuitable. Include the browser-like headers, construct the matching flat-search referer, and use `-N` to avoid buffering SSE output.

```bash
curl -N --max-time 90 -sS \
  -H 'Content-Type: application/json' \
  -H 'Accept: */*' \
  -H 'Origin: https://flysoar.ai' \
  -H 'Referer: https://flysoar.ai/flights/sfo/jfk/YYMMDD/' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36' \
  -X POST 'https://flysoar.ai/api/search/stream' \
  --data '{"origin":"SFO","destination":"JFK","date":"YYYY-MM-DD","cabin":"economy","passengers":1}'
```

For `slices` requests, use `https://flysoar.ai/` as the referer and send the same JSON format accepted by `--input`:

```json
{
  "slices": [
    {"origin": "SFO", "destination": "JFK", "departure_date": "YYYY-MM-DD"},
    {"origin": "LHR", "destination": "SFO", "departure_date": "YYYY-MM-DD"}
  ],
  "cabin": "economy",
  "passengers": 1
}
```

Read complete event blocks, collect JSON from `event: offer`, stop on `event: done`, and surface `event: error`. Do not treat a successful HTTP status as a successful search until `done` arrives.

The flat request format supports one-way and round-trip searches with `origin`, `destination`, `date`, optional `return_date`, `cabin`, and `passengers`; use it for no more than two legs. The `slices` format supports one or more `{origin, destination, departure_date}` legs, including open-jaw and multi-city itineraries; use it for three or more legs.

| Behavior | Implication |
| --- | --- |
| Prices are returned in USD | Do not promise currency conversion; the observed API ignores a requested `currency`. |
| `passengers` is a total-party count | Treat prices as totals for that count; values at or below zero behave as one, and larger counts can reduce available inventory. |
| `direct_only` is ignored by the API | Use `--nonstop-only`. |
| `max_connections` can hang the API | Do not send it, including through the curl fallback. |
| Offers can expire | Preserve `id` and `expires_at` when using raw offers for a shortlist. |

The stream emits `created` (acknowledgement), `batch` (progress), `offer` (flight), `done` (completion), and `error` (search failure). `created` identifies the search and its batch count; `batch` reports elapsed time and new/remaining offer counts. `done` can report `partial`, `error_count`, and a per-provider stop reason. Treat a `done` event marked partial as partial results. The CLI handles `created` and `batch` internally, returns offers on `done`, and reports an `error` as a command failure.

| Scenario | Underlying response | CLI behavior |
| --- | --- | --- |
| Missing required flat fields, invalid date format, or invalid cabin | HTTP 400 JSON | Exit with the API error. |
| Invalid IATA code or past date | HTTP 200 SSE `error` | Exit with the SSE error. |
| Timeout or interrupted stream | Incomplete SSE stream | Treat any returned offers as partial and label them accordingly. |
| Omitted cabin | HTTP 200 SSE | Search economy by default. |

## Raw offer schema

Use `--raw` when a request requires fields not in the normalized output. Treat amounts as decimal strings and nullable fields as optional. `requested_cabin` is the user's requested class; `search_cabin` can differ when exact inventory is unavailable.

```text
offer
├── id, provider, requested_cabin, search_cabin
├── total_amount, total_currency, base_amount, base_currency
├── tax_amount, tax_currency, expires_at, total_emissions_kg
├── conditions
│   ├── refundable, refund_penalty_amount, refund_penalty_currency
│   └── changeable, change_penalty_amount, change_penalty_currency
└── slices[]
    ├── origin, origin_name, origin_city, origin_country_code
    ├── destination, destination_name, destination_city, destination_country_code
    ├── duration, fare_brand, cabin_class
    └── segments[]
        ├── carrier_iata, carrier_name, carrier_logo
        ├── operating_carrier_iata, operating_carrier_name, is_codeshare
        ├── flight_number, operating_flight_number
        ├── departure, arrival, departure_display, arrival_display
        ├── origin, origin_name, origin_city, origin_country_code, origin_time_zone, origin_terminal
        ├── destination, destination_name, destination_city, destination_country_code, destination_time_zone, destination_terminal
        ├── aircraft, duration, cabin_class, fare_basis_code, fare_brand
        ├── baggage_carry_on, baggage_checked
        └── amenities
            ├── wifi {available, cost, provider, speed, partial}
            ├── power {available}
            └── source
```

Treat this schema and the API-behavior guidance above as observed behavior, not a compatibility guarantee. Prefer live CLI output when it differs from these observations.
