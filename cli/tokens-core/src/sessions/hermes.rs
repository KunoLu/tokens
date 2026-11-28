//! Hermes Agent session parser
//!
//! Parses aggregated session rows from Hermes Agent's SQLite state database:
//! - `~/.hermes/state.db`
//! - `$HERMES_HOME/state.db`
//!
//! Hermes stores usage only as lifetime totals per session (and per model in
//! `session_model_usage`), never per API call. Each total is spread across the
//! unambiguous assistant messages in its time window, so a session that runs
//! for days books its usage on the days it was used rather than the day it
//! started (missuo/tokens#129).

use super::utils::{
    open_readonly_sqlite, resolved_provider, sqlite_for_each_row_on, timestamp_secs_to_ms,
};
use super::UnifiedMessage;
use crate::TokenBreakdown;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::warn;

const HERMES_AGENT_NAME: &str = "Hermes Agent";

/// Stands in for a NULL `billing_provider` inside a per-model dedup key. Angle
/// brackets keep it out of the slug space real provider ids live in.
const NULL_PROVIDER_KEY: &str = "<null>";

/// Per-model breakdown, available once Hermes started recording
/// `session_model_usage`.
///
/// Grouping keeps `billing_provider` because the composite primary key lets a
/// single (session, model) pair span several providers; collapsing them would
/// erase one provider from the breakdown and credit its tokens to the other.
/// Cost is resolved per row (actual when non-zero, otherwise estimated) before
/// the SUM, so a reconciled sibling row cannot discard estimate-only siblings.
/// `ORDER BY` sorts the row matching `sessions.model` first within each session
/// and is otherwise total, so the caller can hand `sessions.message_count` to
/// the first row it sees per session and get the primary model whenever one
/// survives grouping — and a deterministic row when none does.
const PER_MODEL_QUERY: &str = r#"
        SELECT
            smu.session_id,
            smu.model,
            smu.billing_provider,
            s.started_at,
            COALESCE(s.message_count, 0) AS message_count,
            SUM(smu.input_tokens)        AS input_tokens,
            SUM(smu.output_tokens)       AS output_tokens,
            SUM(smu.cache_read_tokens)   AS cache_read_tokens,
            SUM(smu.cache_write_tokens)  AS cache_write_tokens,
            SUM(smu.reasoning_tokens)    AS reasoning_tokens,
            SUM(COALESCE(NULLIF(smu.actual_cost_usd, 0), smu.estimated_cost_usd, 0)) AS cost_usd,
            MIN(smu.first_seen)          AS first_seen,
            MAX(smu.last_seen)           AS last_seen
        FROM session_model_usage smu
        JOIN sessions s ON s.id = smu.session_id
        WHERE smu.model IS NOT NULL
          AND TRIM(smu.model) != ''
        GROUP BY smu.session_id, smu.model, smu.billing_provider, s.started_at, s.message_count
        HAVING SUM(smu.input_tokens) > 0
            OR SUM(smu.output_tokens) > 0
            OR SUM(smu.cache_read_tokens) > 0
            OR SUM(smu.cache_write_tokens) > 0
            OR SUM(smu.reasoning_tokens) > 0
            OR SUM(COALESCE(NULLIF(smu.actual_cost_usd, 0), smu.estimated_cost_usd, 0)) > 0
        ORDER BY smu.session_id,
                 CASE WHEN smu.model = s.model THEN 0 ELSE 1 END,
                 smu.model,
                 smu.billing_provider
"#;

/// Session-level totals credited to `sessions.model`. Covers Hermes builds that
/// predate `session_model_usage` and, on builds that have it, sessions with no
/// child rows in it; column order matches `PER_MODEL_QUERY`. `sessions` has no
/// usage window, so the whole session from `started_at` on is the window.
const SESSION_TOTALS_QUERY: &str = r#"
        SELECT
            id,
            model,
            billing_provider,
            started_at,
            message_count,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
            reasoning_tokens,
            COALESCE(actual_cost_usd, estimated_cost_usd, 0) AS cost_usd,
            NULL AS first_seen,
            NULL AS last_seen
        FROM sessions
        WHERE model IS NOT NULL
          AND TRIM(model) != ''
          AND (
            COALESCE(input_tokens, 0) > 0 OR
            COALESCE(output_tokens, 0) > 0 OR
            COALESCE(cache_read_tokens, 0) > 0 OR
            COALESCE(cache_write_tokens, 0) > 0 OR
            COALESCE(reasoning_tokens, 0) > 0 OR
            COALESCE(actual_cost_usd, estimated_cost_usd, 0) > 0
          )
"#;

/// Every assistant message time per session, the only per-call time signal
/// Hermes keeps. `messages` has carried `session_id`, `role` and `timestamp`
/// since its first schema. Compaction re-inserts surviving rows as clones with
/// the original timestamp, so DISTINCT keeps one API call from counting once
/// per generation. Inactive (compacted or rewound) rows stay: each still
/// stands for a call that was billed.
const ASSISTANT_TIMESTAMPS_QUERY: &str = r#"
        SELECT DISTINCT session_id, timestamp
        FROM messages
        WHERE role = 'assistant'
          AND typeof(timestamp) IN ('integer', 'real')
        ORDER BY session_id, timestamp
"#;

/// One usage row, decoded from either query — both project the same columns.
struct HermesUsageRow {
    session_id: String,
    model_id: String,
    billing_provider: Option<String>,
    started_at: f64,
    message_count: i32,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
    reasoning: i64,
    cost: f64,
    /// When Hermes first and last recorded usage for this row (epoch seconds);
    /// NULL for session totals.
    first_seen: Option<f64>,
    last_seen: Option<f64>,
}

fn decode_usage_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HermesUsageRow> {
    Ok(HermesUsageRow {
        session_id: row.get(0)?,
        model_id: row.get(1)?,
        billing_provider: row.get(2)?,
        started_at: row.get(3)?,
        message_count: row.get::<_, Option<i32>>(4)?.unwrap_or(0),
        input: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
        output: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
        cache_read: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
        cache_write: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
        reasoning: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
        cost: row.get::<_, Option<f64>>(10)?.unwrap_or(0.0),
        first_seen: row.get(11)?,
        last_seen: row.get(12)?,
    })
}

/// Runs one of the usage queries. Returns `None` when the query cannot run at
/// all (prepare or execute failure) so the caller can fall back, and `Some` —
/// possibly empty — when it ran and simply matched nothing.
fn query_usage_rows(db_path: &Path, conn: &Connection, query: &str) -> Option<Vec<HermesUsageRow>> {
    let mut rows = Vec::new();
    let scan = sqlite_for_each_row_on(conn, db_path, query, Some("Hermes session"), &mut |row| {
        rows.push(decode_usage_row(row)?);
        Ok(())
    });
    scan.ran().then_some(rows)
}

/// Assistant message times per session, ascending. Empty when the table is
/// missing or unreadable, which leaves every row on its fallback time.
fn query_assistant_timestamps(db_path: &Path, conn: &Connection) -> HashMap<String, Vec<f64>> {
    let mut timestamps: HashMap<String, Vec<f64>> = HashMap::new();
    let scan = sqlite_for_each_row_on(
        conn,
        db_path,
        ASSISTANT_TIMESTAMPS_QUERY,
        Some("Hermes message timestamps"),
        &mut |row| {
            timestamps.entry(row.get(0)?).or_default().push(row.get(1)?);
            Ok(())
        },
    );
    if scan.ran() {
        timestamps
    } else {
        HashMap::new()
    }
}

fn usage_bounds(row: &HermesUsageRow) -> (f64, f64) {
    (
        row.first_seen.unwrap_or(row.started_at),
        row.last_seen.unwrap_or(f64::INFINITY),
    )
}

/// Bound usage to the recorded interval so rotated sessions do not spread
/// onto parent messages carried over with their original timestamps.
fn usage_window<'a>(row: &HermesUsageRow, timestamps: &'a [f64]) -> &'a [f64] {
    let (lo, hi) = usage_bounds(row);
    let start = timestamps.partition_point(|&t| t < lo);
    let end = timestamps.partition_point(|&t| t <= hi).max(start);
    &timestamps[start..end]
}

/// The message table does not identify a model. When models switch back and
/// forth, the same timestamp can fall inside more than one model's interval.
/// Exclude it from all rows instead of attributing it to each model.
fn exclude_ambiguous_times(
    rows: &[HermesUsageRow],
    timestamps: &mut Vec<f64>,
    overlaps: &mut Vec<(f64, f64)>,
) {
    overlaps.clear();
    for (i, row) in rows.iter().enumerate().skip(1) {
        let (start, end) = usage_bounds(row);
        for other in &rows[..i] {
            if row.model_id == other.model_id {
                continue;
            }
            let (other_start, other_end) = usage_bounds(other);
            let start = start.max(other_start);
            let end = end.min(other_end);
            if start <= end {
                overlaps.push((start, end));
            }
        }
    }
    if overlaps.is_empty() {
        return;
    }

    overlaps.sort_unstable_by(|left, right| left.0.total_cmp(&right.0));
    let mut overlaps = overlaps.iter();
    let mut current = overlaps.next();
    timestamps.retain(|&time| {
        while let Some(&(start, end)) = current {
            if time < start {
                return true;
            }
            if time <= end {
                return false;
            }
            current = overlaps.next();
        }
        true
    });
}

/// Piece `index` of `total` split into `pieces` near-equal integer shares.
/// Differences of cumulative floors sum back to exactly `total`.
fn integer_share(total: i64, index: usize, pieces: usize) -> i64 {
    let cumulative = |k: usize| (i128::from(total) * k as i128 / pieces as i128) as i64;
    cumulative(index + 1) - cumulative(index)
}

/// Splits one usage row evenly across the remaining assistant times inside its
/// window. Without a reliable time, keep it whole at `first_seen`, or at
/// `started_at` when the first observation is unknown.
fn push_spread_messages(
    row: HermesUsageRow,
    row_key: Option<&str>,
    timestamps: &[f64],
    messages: &mut Vec<UnifiedMessage>,
) {
    let fallback = [row.first_seen.unwrap_or(row.started_at)];
    let window = usage_window(&row, timestamps);
    let times = if window.is_empty() {
        &fallback[..]
    } else {
        window
    };
    let pieces = times.len();
    let row_key = row_key.unwrap_or(row.session_id.as_str());

    let provider = resolved_provider(row.billing_provider, &row.model_id, "hermes");
    let tokens = TokenBreakdown {
        input: row.input.max(0),
        output: row.output.max(0),
        cache_read: row.cache_read.max(0),
        cache_write: row.cache_write.max(0),
        reasoning: row.reasoning.max(0),
    };
    let cost = row.cost.max(0.0);
    let cumulative_cost = |k: usize| cost * k as f64 / pieces as f64;

    for (index, &timestamp) in times.iter().enumerate() {
        let piece_tokens = TokenBreakdown {
            input: integer_share(tokens.input, index, pieces),
            output: integer_share(tokens.output, index, pieces),
            cache_read: integer_share(tokens.cache_read, index, pieces),
            cache_write: integer_share(tokens.cache_write, index, pieces),
            reasoning: integer_share(tokens.reasoning, index, pieces),
        };
        let piece_cost = if index + 1 == pieces {
            cost - cumulative_cost(index)
        } else {
            cumulative_cost(index + 1) - cumulative_cost(index)
        };
        // `sessions.message_count` stays on one piece, as it stayed on one
        // row: per-day message counts are unchanged, and the other pieces stay
        // count-neutral so time metrics still see one session.
        let message_count = if index == 0 {
            row.message_count.max(0)
        } else {
            0
        };
        if piece_tokens.total() == 0 && piece_cost <= 0.0 && message_count == 0 {
            continue;
        }

        let mut msg = UnifiedMessage::new_with_agent(
            "hermes",
            row.model_id.clone(),
            provider.clone(),
            row.session_id.clone(),
            timestamp_secs_to_ms(timestamp),
            piece_tokens,
            piece_cost.max(0.0),
            Some(HERMES_AGENT_NAME.to_string()),
        );
        msg.message_count = message_count;
        msg.dedup_key = Some(format!("{row_key}#{index}"));
        messages.push(msg);
    }
}

fn has_session_model_usage(db_path: &Path, conn: &Connection) -> bool {
    match conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'session_model_usage'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(_) => true,
        Err(rusqlite::Error::QueryReturnedNoRows) => false,
        Err(err) => {
            warn!(
                db_path = %db_path.display(),
                error = %err,
                "Failed to probe Hermes session_model_usage table; using session totals"
            );
            false
        }
    }
}

pub fn parse_hermes_sqlite(db_path: &Path) -> Vec<UnifiedMessage> {
    parse_hermes_databases(&[db_path])
}

enum SessionSource {
    PerModel,
    Totals(usize), // index in pending_totals
}

/// Parses every Hermes database, preferring per-model usage over session
/// totals for the same session regardless of database order. The first
/// database wins for duplicate rows within a representation, before spreading
/// makes copies with different message histories produce different pieces.
pub fn parse_hermes_databases<P: AsRef<Path>>(db_paths: &[P]) -> Vec<UnifiedMessage> {
    let mut session_sources = HashMap::new();
    let mut seen_model_rows = HashSet::new();
    let mut pending_totals = Vec::new();
    let mut overlaps = Vec::new();
    let mut messages = Vec::new();
    for db_path in db_paths {
        let db_path = db_path.as_ref();
        let Some(conn) = open_hermes_database(db_path) else {
            continue;
        };
        let model_rows = query_per_model_usage(db_path, &conn);
        let totals = query_usage_rows(db_path, &conn, SESSION_TOTALS_QUERY).unwrap_or_default();
        if model_rows.is_empty() && totals.is_empty() {
            continue;
        }
        let mut timestamps = query_assistant_timestamps(db_path, &conn);
        for rows in model_rows
            .chunk_by(|a, b| a.session_id == b.session_id)
            .filter(|rows| rows.len() > 1)
        {
            if let Some(times) = timestamps.get_mut(&rows[0].session_id) {
                exclude_ambiguous_times(rows, times, &mut overlaps);
            }
        }

        for mut row in model_rows {
            let provider = row.billing_provider.as_deref().unwrap_or(NULL_PROVIDER_KEY);
            let row_key = format!("hermes:{}:{}:{}", row.session_id, row.model_id, provider);
            if seen_model_rows.contains(&row_key) {
                continue;
            }
            // SQL orders the primary model first. Only the first per-model
            // row of the session carries the session's message count.
            if let Some(source) = session_sources.get_mut(&row.session_id) {
                match *source {
                    SessionSource::PerModel => row.message_count = 0,
                    SessionSource::Totals(index) => {
                        pending_totals[index] = None;
                        *source = SessionSource::PerModel;
                    }
                }
            } else {
                session_sources.insert(row.session_id.clone(), SessionSource::PerModel);
            }
            let times = timestamps.get(&row.session_id).map_or(&[][..], Vec::as_slice);
            push_spread_messages(row, Some(&row_key), times, &mut messages);
            seen_model_rows.insert(row_key);
        }

        // Keep only the first totals row, and defer spreading: this session
        // may still have per-model usage in a later database.
        for row in totals {
            if session_sources.contains_key(&row.session_id) {
                continue;
            }
            session_sources.insert(
                row.session_id.clone(),
                SessionSource::Totals(pending_totals.len()),
            );
            let times = timestamps.remove(&row.session_id).unwrap_or_default();
            pending_totals.push(Some((row, times)));
        }
    }
    for (row, times) in pending_totals.into_iter().flatten() {
        push_spread_messages(row, None, &times, &mut messages);
    }
    messages
}

fn open_hermes_database(db_path: &Path) -> Option<Connection> {
    match open_readonly_sqlite(db_path) {
        Ok(conn) => Some(conn),
        Err(err) => {
            warn!(
                db_path = %db_path.display(),
                error = %err,
                "Failed to open Hermes state database"
            );
            None
        }
    }
}

fn query_per_model_usage(db_path: &Path, conn: &Connection) -> Vec<HermesUsageRow> {
    // Hermes builds that predate `session_model_usage` cannot run the per-model
    // query at all; probing first keeps that expected case quiet instead of
    // logging a prepare failure for every such install. When the table does
    // exist but the query still fails to prepare — schema drift, e.g. an older
    // `session_model_usage` without `reasoning_tokens` — `query_usage_rows`
    // returns None and the session totals below carry the whole database, so
    // drift degrades to coarser data instead of losing all of it.
    if has_session_model_usage(db_path, conn) {
        query_usage_rows(db_path, conn, PER_MODEL_QUERY).unwrap_or_default()
    } else {
        Vec::new()
    }
}
