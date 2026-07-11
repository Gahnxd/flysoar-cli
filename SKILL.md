# FlySoar API Skill

## Overview

Search for live flight offers using FlySoar.ai's anonymous public SSE endpoint. No accounts, API keys, or authentication required.

## Endpoint

```
POST https://flysoar.ai/api/search/stream
```

Returns Server-Sent Events (SSE). Stream `event: offer` lines until `event: done`.

## Required Headers

```
Content-Type: application/json
Accept: */*
Origin: https://flysoar.ai
Referer: https://flysoar.ai/flights/<origin>/<dest>/<yyMMdd>/[<return yyMMdd>/]
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36
```

## Request Formats

### Flat format — one-way

```json
{
  "origin": "SFO",
  "destination": "JFK",
  "date": "2026-07-15",
  "cabin": "economy",
  "passengers": 1
}
```

### Flat format — round-trip

```json
{
  "origin": "SFO",
  "destination": "JFK",
  "date": "2026-07-15",
  "return_date": "2026-07-22",
  "cabin": "economy",
  "passengers": 1
}
```

### Slices format — multi-city / open-jaw

```json
{
  "slices": [
    {"origin": "SFO", "destination": "JFK", "departure_date": "2026-07-15"},
    {"origin": "LHR", "destination": "SFO", "departure_date": "2026-07-22"}
  ],
  "cabin": "economy",
  "passengers": 1
}
```

Supports 1+ slices. Use for one-way, round-trip, open-jaw, or multi-city itineraries.

## Parameters

| Parameter | Format | Required | Default | Notes |
|-----------|--------|----------|---------|-------|
| `origin` | IATA code (airport or city) | flat: yes | — | e.g. `SFO`, `JFK`, `NYC`, `LON` |
| `destination` | IATA code (airport or city) | flat: yes | — | Same as origin |
| `date` | `YYYY-MM-DD` | flat: yes | — | Must be future date |
| `return_date` | `YYYY-MM-DD` | flat: no | — | Omit for one-way |
| `slices` | array of `{origin, destination, departure_date}` | slices: yes | — | Alternative to flat fields |
| `cabin` | string | no | `economy` | `economy`, `premium_economy`, `business`, `first` |
| `passengers` | integer | no | `1` | Values ≤ 0 treated as 1. Prices scale linearly. |
| `currency` | string | no | `USD` | **Ignored** — all prices always in USD |

### Unsupported parameters

- `max_connections` — causes server hang; do not send
- `direct_only` — silently ignored; filter stops client-side

## Response: SSE Events

### `event: created`

Search request acknowledged. Contains `id`, `total_batches`, `is_multi_city`.

### `event: batch`

Progress updates during search. Contains `batch_index`, `remaining_batches`, `new_offer_count`.

### `event: offer`

Flight offer. This is the main payload to collect.

### `event: done`

Search complete. Contains `offer_count`, `stopped_because` (`complete` or `partial`).

### `event: error`

Search-level error. Contains `error` (string) and `status` (int).

## Offer Schema

```
offer
├── id                          string   e.g. "off_0000B8DDGR9hsD6DoHQ7dv"
├── provider                    string   e.g. "duffel"
├── requested_cabin             string
├── search_cabin                string
├── total_amount                string   e.g. "368.40"
├── total_currency              string   always "USD"
├── base_amount                 string
├── base_currency               string
├── tax_amount                  string
├── tax_currency                string
├── expires_at                  string   ISO-8601 timestamp
├── total_emissions_kg          integer
├── conditions
│   ├── refundable              bool|null
│   ├── refund_penalty_amount   string|null
│   ├── changeable              bool|null
│   └── change_penalty_amount   string|null
└── slices[]
    ├── origin                  string   IATA code
    ├── origin_name             string
    ├── origin_city             string
    ├── destination             string   IATA code
    ├── destination_name        string
    ├── destination_city        string
    ├── duration                string   ISO-8601 duration e.g. "PT5H48M"
    ├── fare_brand              string|null
    ├── cabin_class             string   e.g. "Economy", "Business", "First"
    └── segments[]
        ├── carrier_iata              string   e.g. "AA"
        ├── carrier_name              string   e.g. "American Airlines"
        ├── carrier_logo              string   URL
        ├── operating_carrier_iata    string
        ├── operating_carrier_name    string
        ├── is_codeshare              bool
        ├── flight_number             string
        ├── operating_flight_number   string
        ├── departure                 string   ISO-8601
        ├── arrival                   string   ISO-8601
        ├── departure_display         string   e.g. "1:38 PM PST"
        ├── arrival_display           string
        ├── origin                    string   IATA code
        ├── origin_name               string
        ├── origin_city               string
        ├── origin_time_zone          string   IANA tz
        ├── origin_terminal           string|null
        ├── destination               string   IATA code
        ├── destination_name          string
        ├── destination_city          string
        ├── destination_time_zone     string
        ├── destination_terminal      string|null
        ├── aircraft                  string
        ├── duration                  string   ISO-8601 duration
        ├── cabin_class               string
        ├── fare_basis_code           string
        ├── fare_brand                string|null
        ├── baggage_carry_on          int|null
        ├── baggage_checked           int|null
        └── amenities
            ├── wifi    {available: bool, cost: "free"|"paid"|null, partial: bool}
            ├── power   {available: bool}
            └── source   string
```

## Error Handling

| Scenario | HTTP | Format | Body |
|----------|------|--------|------|
| Missing `origin`/`destination`/`date` | 400 | JSON | `{"error":"origin, destination, and date are required"}` |
| Invalid date format | 400 | JSON | `{"error":"date must be YYYY-MM-DD"}` |
| Invalid cabin | 400 | JSON | `{"error":"invalid cabin"}` |
| Invalid IATA code | 200 | SSE | `event: error` → `{"error":"Invalid IATA code","status":422}` |
| Past date | 200 | SSE | `event: error` → `{"error":"Invalid date","status":422}` |

**Pattern:** Missing/invalid required fields → `HTTP 400` with plain JSON. Valid request with invalid data (bad IATA, past date) → `HTTP 200` then `event: error` in the SSE stream.

## Usage Examples

### curl — one-way economy

```bash
curl -N -s \
  -H "Content-Type: application/json" \
  -H "Accept: */*" \
  -H "Origin: https://flysoar.ai" \
  -H "Referer: https://flysoar.ai/flights/sfo/jfk/260715/" \
  -H "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36" \
  -X POST https://flysoar.ai/api/search/stream \
  -d '{"origin":"SFO","destination":"JFK","date":"2026-07-15","cabin":"economy","passengers":1}'
```

### curl — multi-city via slices

```bash
curl -N -s \
  -H "Content-Type: application/json" \
  -H "Accept: */*" \
  -H "Origin: https://flysoar.ai" \
  -H "Referer: https://flysoar.ai/" \
  -H "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36" \
  -X POST https://flysoar.ai/api/search/stream \
  -d '{"slices":[{"origin":"SFO","destination":"JFK","departure_date":"2026-07-15"},{"origin":"LHR","destination":"SFO","departure_date":"2026-07-22"}],"cabin":"economy","passengers":1}'
```

### Python — parse SSE offers

```python
import json
import requests

resp = requests.post(
    "https://flysoar.ai/api/search/stream",
    json={"origin": "SFO", "destination": "JFK", "date": "2026-07-15",
          "cabin": "economy", "passengers": 1},
    headers={
        "Content-Type": "application/json",
        "Accept": "*/*",
        "Origin": "https://flysoar.ai",
        "Referer": "https://flysoar.ai/flights/sfo/jfk/260715/",
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    },
    stream=True,
    timeout=90,
)

offers = []
current_event = None
for line in resp.iter_lines(decode_unicode=True):
    line = line.strip() if line else ""
    if not line:
        current_event = None
        continue
    if line.startswith("event:"):
        current_event = line.split(":", 1)[1].strip()
        if current_event == "done":
            break
    elif line.startswith("data:") and current_event == "offer":
        offers.append(json.loads(line.split("data:", 1)[1].strip()))

print(f"Got {len(offers)} offers")
for o in offers[:5]:
    s = o["slices"][0]
    seg = s["segments"][0]
    print(f"  ${o['total_amount']} {seg['carrier_iata']}{seg['flight_number']} "
          f"{seg['origin']}→{seg['destination']} {seg['departure'][:16]}")
```

## Tips

- Use `-N` (no buffer) with curl to stream SSE as it arrives.
- City codes (`NYC`, `LON`) work but same-metro pairs (e.g. `NYC`→`JFK`) return 0 offers.
- Stop filtering must be done client-side — count `len(slices[i].segments) - 1`.
- Sort offers by `float(offer["total_amount"])` ascending for cheapest first.
- Offers expire — check `expires_at` before use.
- The `Referer` header format: `/flights/<origin_lower>/<dest_lower>/<yyMMdd>/[<return_yyMMdd>/]`. For multi-city, just use `https://flysoar.ai/`.
