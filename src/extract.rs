use std::collections::HashMap;

use color_eyre::eyre::{Result, WrapErr};
use geo_types::Point;
use gpx::{Gpx, GpxVersion, Time, Track, TrackSegment, Waypoint};
use rusqlite::Connection;
use time::{macros::datetime, Duration, OffsetDateTime, PrimitiveDateTime};

const ZERO_DATE: PrimitiveDateTime = datetime!(2001-01-01 00:00:00);

fn convert_time(time: f64) -> OffsetDateTime {
    let duration = Duration::seconds_f64(time);
    (ZERO_DATE + duration).assume_utc()
}

#[derive(Debug)]
pub struct TrackRow {
    id: i64,
    name: String,
    time: f64,
}

#[derive(Debug)]
pub struct TrackPoint {
    track_id: i64,
    lat: f64,
    lon: f64,
    ele: f64,
    time: f64,
}

pub fn extract(sqlite: &Connection, track_name: Option<String>) -> Result<Gpx> {
    let mut tracks_stmt = sqlite
        .prepare("SELECT Z_PK, ZNAME, ZDATE FROM ZTRACK")
        .wrap_err("Failed to prepare for tracks statement")?;

    let tracks = tracks_stmt
        .query_map([], |row| {
            Ok(TrackRow {
                id: row.get(0)?,
                name: row.get(1)?,
                time: row.get(2)?,
            })
        })
        .wrap_err("Failed to query for tracks")?
        .collect::<rusqlite::Result<Vec<TrackRow>>>()
        .wrap_err("Failed to collect tracks")?
        .into_iter()
        .filter(|track| {
            if let Some(name) = &track_name {
                &track.name == name
            } else {
                true
            }
        })
        .collect::<Vec<TrackRow>>();

    let track_ids = tracks
        .iter()
        .map(|track| track.id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let mut trackpoints_stmt = sqlite
        .prepare(&format!(
            "
            SELECT
                ZTRACK,
                ZLATITUDE,
                ZLONGITUDE,
                ZALTITUDE,
                ZDATE
            FROM ZCOURSEPOINT
            WHERE ZTRACK IN ({track_ids})
            ORDER BY ZDATE"
        ))
        .wrap_err("Failed to prepare for trackpoints statement")?;

    let trackpoints = trackpoints_stmt
        .query_map([], |row| {
            Ok(TrackPoint {
                track_id: row.get(0)?,
                lat: row.get(1)?,
                lon: row.get(2)?,
                ele: row.get(3)?,
                time: row.get(4)?,
            })
        })
        .wrap_err("Failed to query for trackpoints")?
        .collect::<rusqlite::Result<Vec<TrackPoint>>>()
        .wrap_err("Failed to collect trackpoints")?;

    let mut track_segments = HashMap::new();
    for db_track in &tracks {
        track_segments.insert(db_track.id, TrackSegment::new());
    }

    let mut current_track_id = tracks[0].id;
    let mut current_track = track_segments
        .get_mut(&current_track_id)
        .expect("No track found");

    for track_point in trackpoints {
        let mut point = Waypoint::new(Point::new(track_point.lon, track_point.lat));
        point.elevation = Some(track_point.ele);
        point.time = Some(Time::from(convert_time(track_point.time)));

        if track_point.track_id != current_track_id {
            current_track_id = track_point.track_id;
            current_track = track_segments
                .get_mut(&current_track_id)
                .expect("No track found");
        }

        current_track.points.push(point);
    }

    let mut gpx = Gpx::default();
    for db_track in tracks {
        let mut track = Track::new();
        track.name = Some(db_track.name);
        track
            .segments
            .push(track_segments.get(&db_track.id).unwrap().clone());
        track.description = Some(format!("Track Timestamp: {}", convert_time(db_track.time)));
        gpx.tracks.push(track);
    }

    gpx.version = GpxVersion::Gpx11;
    Ok(gpx)
}
