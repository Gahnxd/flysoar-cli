use comfy_table::{Cell, Color, ContentArrangement, Table};

use crate::models::SearchResult;

pub fn print_json(result: &SearchResult) {
    let json = serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string());
    println!("{}", json);
}

pub fn print_csv(result: &SearchResult) {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record([
        "price",
        "currency",
        "stops",
        "airline",
        "flight_numbers",
        "depart_time",
        "arrive_time",
        "duration",
        "cabin_class",
        "route",
        "emissions_kg",
    ])
    .unwrap();

    for offer in &result.offers {
        let route = offer
            .slices
            .iter()
            .map(|s| format!("{}->{}", s.origin, s.destination))
            .collect::<Vec<_>>()
            .join(" | ");

        wtr.write_record([
            format!("{:.2}", offer.price),
            offer.currency.clone(),
            offer.stops.to_string(),
            offer.airline.clone(),
            offer.flight_numbers.clone(),
            offer.depart_time.clone(),
            offer.arrive_time.clone(),
            offer.duration.clone(),
            offer.cabin_class.clone(),
            route,
            offer
                .emissions_kg
                .map(|e| e.to_string())
                .unwrap_or_default(),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}

pub fn print_table(result: &SearchResult) {
    if result.offers.is_empty() {
        println!("No offers found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Price").fg(Color::Cyan),
            Cell::new("Airline").fg(Color::Cyan),
            Cell::new("Flight").fg(Color::Cyan),
            Cell::new("Route").fg(Color::Cyan),
            Cell::new("Departure").fg(Color::Cyan),
            Cell::new("Arrival").fg(Color::Cyan),
            Cell::new("Stops").fg(Color::Cyan),
            Cell::new("Duration").fg(Color::Cyan),
            Cell::new("Cabin").fg(Color::Cyan),
        ]);

    for offer in &result.offers {
        let route = offer
            .slices
            .iter()
            .map(|s| format!("{}→{}", s.origin, s.destination))
            .collect::<Vec<_>>()
            .join(" | ");

        let price_str = format!("${:.2} {}", offer.price, offer.currency);

        table.add_row(vec![
            Cell::new(price_str).fg(Color::Green),
            Cell::new(&offer.airline),
            Cell::new(&offer.flight_numbers),
            Cell::new(route),
            Cell::new(&offer.depart_time),
            Cell::new(&offer.arrive_time),
            Cell::new(offer.stops),
            Cell::new(&offer.duration),
            Cell::new(&offer.cabin_class),
        ]);
    }

    println!("{table}");
    println!("\n{} offers found.", result.count);
}
