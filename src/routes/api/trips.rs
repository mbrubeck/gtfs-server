use super::model_api::tripresult::TripResult;
use super::model_api::meta::Meta;

use models::trip::Trip;

use super::super::RoutesHandler;
use super::super::Json;
use super::super::State;
use super::super::Pool;
use super::super::PostgresConnectionManager;

use postgres::rows::Row;

#[get("/trips/<stop_id>")]
pub fn trips_stopid(rh: State<RoutesHandler>, stop_id: String) -> Json<TripResult>{
    let query =
        "SELECT \
        uid,\
        route_id,\
        service_id,\
        trip_id,\
        headsign,\
        short_name,\
        direction_id,\
        feed_id \
        FROM trip \
        WHERE trip_id IN \
        (SELECT trip_id FROM stop_time WHERE \
            stop_id=(SELECT stop.id FROM stop WHERE uid=$1) \
            GROUP BY trip_id \
        ) LIMIT 50";

    let conn = rh.pool.clone().get().unwrap();
    let trips = conn.query(
        query,
        &[
            &stop_id
        ]
    );

    let mut trips_result: Vec<Trip> = Vec::new();

    for row in trips.expect("Query failed").iter() {
        let route = parse_trip_row(&row);
        trips_result.push(route);
    }

    let rr = TripResult {
        result: trips_result,
        meta: Meta{
            success: true,
            error: Option::None
        }
    };

    Json(rr)
}

fn parse_trip_row(row: &Row) -> Trip {
    let mut t = Trip::new(
        row.get(0),
        row.get(1),
        row.get(2),
        row.get(4),
        row.get(5),
        row.get(6),
    );
    t.set_id(row.get(3));
    t.set_feed_id(row.get(7));
    t
}