use super::model_api::stopresult::StopResult;
use super::model_api::stopresult::StopDistance;
use super::model_api::stopresult::StopDistanceResult;
use super::model_api::meta::Meta;
use super::model_api::error::Error;

use models::stop::Stop;

use super::super::RoutesHandler;
use super::super::Json;
use super::super::State;
use super::super::Pool;
use super::super::PostgresConnectionManager;
use postgres::rows::Row;

#[get("/stops")]
pub fn stops(rh: State<RoutesHandler>) -> Json<StopResult> {

    let sr = StopResult {
        result: get_stops(&rh.pool),
        meta: Meta{
            success: true,
            error: Option::None
        }
    };
    Json(sr)
}

#[get("/stops/near/<lat>/<lng>/<meters>")]
pub fn stops_near(rh: State<RoutesHandler>, lat: f32, lng: f32, meters: f64) -> Json<StopDistanceResult> {

    let sr = StopDistanceResult {
        result: get_stops_near(&rh.pool, lat, lng, meters),
        meta: Meta{
            success: true,
            error: Option::None
        }
    };
    Json(sr)
}

fn parse_stop_row(row: &Row) -> Stop {
    let lat : f64 = row.get(6);
    let lng : f64 = row.get(7);

    let uid = row.get(0);
    let id = row.get(1);
    let name = row.get(2);
    let location_type = row.get(3);
    let parent_station : String = row.get(4);
    let parent_station : Option<String> = match &parent_station.as_str() {
        &"" => Option::None,
        s => Option::Some(s.to_string())
    };
    let feed_id = row.get(5);

    let mut stop = Stop::new(
        uid, name, lat, lng, location_type, parent_station
    );

    &stop.set_id(id);
    &stop.set_feed_id(feed_id);

    stop
}

fn get_stops(pool: &Pool<PostgresConnectionManager>) -> Vec<Stop> {
    let query = "SELECT
        uid,
        id,
        name,
        type,
        parent_stop,
        feed_id,
        ST_Y(position::geometry) as lat,
        ST_X(position::geometry) as lng FROM stop
        LIMIT 50";

    let conn = pool.clone().get().unwrap();
    let stops = conn.query(
        query, &[]
    );

    let mut stops_result : Vec<Stop> = Vec::new();

    for row in stops.expect("Query failed").iter() {
        let stop = parse_stop_row(&row);
        stops_result.push(stop);
    }

    stops_result
}

fn get_stops_near(pool: &Pool<PostgresConnectionManager>,
                  lat: f32,
                  lng: f32,
                  meters: f64) -> Vec<StopDistance> {
    let query = "SELECT * FROM (SELECT
        uid,
        id,
        name,
        type,
        parent_stop,
        feed_id,
        ST_Distance(position, \
        ST_GeomFromText($1)) as distance,
        ST_Y(position::geometry) as lat,
        ST_X(position::geometry) as lng FROM stop) as s1 WHERE \
        distance <= $2\
        ORDER BY distance ASC \
        LIMIT 50;";

    //println!(format!("{}", query));
    let conn = pool.clone().get().unwrap();
    let stops = conn.query(
        query,
        &[
            &format!("POINT({:.5} {:.5})", lng, lat),
            &meters
        ]
    );

    let mut stops_result : Vec<StopDistance> = Vec::new();

    for row in stops.expect("Query failed").iter() {
        let stop = parse_stop_row(&row);
        let distance = row.get(6);

        let sd = StopDistance {
            stop,
            distance
        };

        stops_result.push(sd);
    }

    stops_result
}