#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Request models
// ---------------------------------------------------------------------------

fn default_cabin() -> String {
    "economy".to_string()
}

fn default_passengers() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slices: Option<Vec<SliceRequest>>,
    #[serde(default = "default_cabin")]
    pub cabin: String,
    #[serde(default = "default_passengers")]
    pub passengers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceRequest {
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
}

// ---------------------------------------------------------------------------
// Response models (SSE offer payloads)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub requested_cabin: Option<String>,
    #[serde(default)]
    pub search_cabin: Option<String>,
    pub total_amount: String,
    pub total_currency: String,
    #[serde(default)]
    pub base_amount: Option<String>,
    #[serde(default)]
    pub base_currency: Option<String>,
    #[serde(default)]
    pub tax_amount: Option<String>,
    #[serde(default)]
    pub tax_currency: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub total_emissions_kg: Option<i64>,
    #[serde(default)]
    pub conditions: Option<Conditions>,
    pub slices: Vec<Slice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditions {
    #[serde(default)]
    pub refundable: Option<bool>,
    #[serde(default)]
    pub refund_penalty_amount: Option<String>,
    #[serde(default)]
    pub changeable: Option<bool>,
    #[serde(default)]
    pub change_penalty_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice {
    pub origin: String,
    #[serde(default)]
    pub origin_name: Option<String>,
    #[serde(default)]
    pub origin_city: Option<String>,
    pub destination: String,
    #[serde(default)]
    pub destination_name: Option<String>,
    #[serde(default)]
    pub destination_city: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub fare_brand: Option<String>,
    #[serde(default)]
    pub cabin_class: Option<String>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub carrier_iata: String,
    #[serde(default)]
    pub carrier_name: Option<String>,
    #[serde(default)]
    pub carrier_logo: Option<String>,
    #[serde(default)]
    pub operating_carrier_iata: Option<String>,
    #[serde(default)]
    pub operating_carrier_name: Option<String>,
    #[serde(default)]
    pub is_codeshare: Option<bool>,
    pub flight_number: String,
    #[serde(default)]
    pub operating_flight_number: Option<String>,
    pub departure: String,
    pub arrival: String,
    #[serde(default)]
    pub departure_display: Option<String>,
    #[serde(default)]
    pub arrival_display: Option<String>,
    pub origin: String,
    #[serde(default)]
    pub origin_name: Option<String>,
    #[serde(default)]
    pub origin_time_zone: Option<String>,
    #[serde(default)]
    pub origin_terminal: Option<String>,
    pub destination: String,
    #[serde(default)]
    pub destination_name: Option<String>,
    #[serde(default)]
    pub destination_time_zone: Option<String>,
    #[serde(default)]
    pub destination_terminal: Option<String>,
    #[serde(default)]
    pub aircraft: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub cabin_class: Option<String>,
    #[serde(default)]
    pub fare_basis_code: Option<String>,
    #[serde(default)]
    pub fare_brand: Option<String>,
    #[serde(default)]
    pub baggage_carry_on: Option<i32>,
    #[serde(default)]
    pub baggage_checked: Option<i32>,
    #[serde(default)]
    pub amenities: Option<Amenities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amenities {
    #[serde(default)]
    pub wifi: Option<WifiAmenity>,
    #[serde(default)]
    pub power: Option<PowerAmenity>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAmenity {
    #[serde(default)]
    pub available: Option<bool>,
    #[serde(default)]
    pub cost: Option<String>,
    #[serde(default)]
    pub partial: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAmenity {
    #[serde(default)]
    pub available: Option<bool>,
}

// ---------------------------------------------------------------------------
// SSE event wrapper
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct SseError {
    pub error: String,
    #[serde(default)]
    pub status: Option<i32>,
}

// ---------------------------------------------------------------------------
// CLI output models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub query: serde_json::Value,
    pub offers: Vec<OfferSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfferSummary {
    pub price: f64,
    pub currency: String,
    pub slices: Vec<SliceSummary>,
    pub stops: usize,
    pub airline: String,
    pub flight_numbers: String,
    pub depart_time: String,
    pub arrive_time: String,
    pub duration: String,
    pub cabin_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissions_kg: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceSummary {
    pub origin: String,
    pub destination: String,
    pub departure: String,
    pub arrival: String,
    pub stops: usize,
    pub segments: Vec<SegmentSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentSummary {
    pub carrier: String,
    pub carrier_name: String,
    pub flight_number: String,
    pub origin: String,
    pub destination: String,
    pub departure: String,
    pub arrival: String,
    pub aircraft: String,
    pub cabin_class: String,
}

impl Offer {
    pub fn to_summary(&self) -> OfferSummary {
        let price: f64 = self.total_amount.parse().unwrap_or(0.0);
        let slices: Vec<SliceSummary> = self
            .slices
            .iter()
            .map(|s| SliceSummary {
                origin: s.origin.clone(),
                destination: s.destination.clone(),
                departure: s
                    .segments
                    .first()
                    .map(|seg| seg.departure.clone())
                    .unwrap_or_default(),
                arrival: s
                    .segments
                    .last()
                    .map(|seg| seg.arrival.clone())
                    .unwrap_or_default(),
                stops: s.segments.len().saturating_sub(1),
                segments: s
                    .segments
                    .iter()
                    .map(|seg| SegmentSummary {
                        carrier: seg.carrier_iata.clone(),
                        carrier_name: seg
                            .carrier_name
                            .clone()
                            .unwrap_or_else(|| seg.carrier_iata.clone()),
                        flight_number: seg.flight_number.clone(),
                        origin: seg.origin.clone(),
                        destination: seg.destination.clone(),
                        departure: seg.departure.clone(),
                        arrival: seg.arrival.clone(),
                        aircraft: seg.aircraft.clone().unwrap_or_default(),
                        cabin_class: seg
                            .cabin_class
                            .clone()
                            .unwrap_or_else(|| s.cabin_class.clone().unwrap_or_default()),
                    })
                    .collect(),
            })
            .collect();

        let first_slice = &self.slices[0];
        let first_seg = first_slice.segments.first().unwrap();
        let last_seg = first_slice.segments.last().unwrap();
        let stops = first_slice.segments.len().saturating_sub(1);
        let airline = first_seg
            .carrier_name
            .clone()
            .unwrap_or_else(|| first_seg.carrier_iata.clone());
        let flight_numbers = first_slice
            .segments
            .iter()
            .map(|seg| format!("{}{}", seg.carrier_iata, seg.flight_number))
            .collect::<Vec<_>>()
            .join(", ");
        let depart_time = first_seg.departure.clone();
        let arrive_time = last_seg.arrival.clone();
        let duration = first_slice.duration.clone().unwrap_or_default();
        let cabin_class = first_seg
            .cabin_class
            .clone()
            .or_else(|| first_slice.cabin_class.clone())
            .unwrap_or_default();

        OfferSummary {
            price,
            currency: self.total_currency.clone(),
            slices,
            stops,
            airline,
            flight_numbers,
            depart_time,
            arrive_time,
            duration,
            cabin_class,
            emissions_kg: self.total_emissions_kg,
            expires_at: self.expires_at.clone(),
        }
    }
}
