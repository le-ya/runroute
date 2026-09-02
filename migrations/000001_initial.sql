CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE route_searches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    request jsonb NOT NULL,
    start_point geography(PointZ, 4326) NOT NULL,
    destination_point geography(PointZ, 4326),
    status text NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
    seed bigint NOT NULL,
    dataset_version text NOT NULL,
    scoring_version text NOT NULL,
    configuration_version text NOT NULL,
    limits jsonb NOT NULL,
    timing_ms jsonb NOT NULL DEFAULT '{}'::jsonb,
    failure_code text,
    failure_detail text,
    created_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((status = 'failed') OR (failure_code IS NULL AND failure_detail IS NULL)),
    CHECK ((status IN ('completed', 'failed', 'cancelled')) = (finished_at IS NOT NULL))
);

CREATE INDEX route_searches_status_created_idx
    ON route_searches (status, created_at);
CREATE INDEX route_searches_start_gist_idx
    ON route_searches USING gist (start_point);
CREATE INDEX route_searches_destination_gist_idx
    ON route_searches USING gist (destination_point)
    WHERE destination_point IS NOT NULL;

CREATE TABLE generation_jobs (
    search_id uuid PRIMARY KEY REFERENCES route_searches (id) ON DELETE CASCADE,
    state text NOT NULL CHECK (state IN ('available', 'claimed', 'completed', 'failed', 'cancelled')),
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    available_at timestamptz NOT NULL DEFAULT now(),
    claimed_by text,
    claimed_at timestamptz,
    lease_expires_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (state = 'claimed' AND claimed_by IS NOT NULL AND claimed_at IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR
        (state <> 'claimed' AND claimed_by IS NULL AND claimed_at IS NULL AND lease_expires_at IS NULL)
    )
);

CREATE INDEX generation_jobs_available_idx
    ON generation_jobs (available_at, search_id)
    WHERE state = 'available';
CREATE INDEX generation_jobs_expired_lease_idx
    ON generation_jobs (lease_expires_at)
    WHERE state = 'claimed';

CREATE TABLE route_candidates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    search_id uuid NOT NULL REFERENCES route_searches (id) ON DELETE CASCADE,
    rank smallint CHECK (rank > 0),
    geometry geometry(LineStringZ, 4326) NOT NULL,
    metrics jsonb NOT NULL,
    total_score smallint NOT NULL CHECK (total_score BETWEEN 0 AND 100),
    score_breakdown jsonb NOT NULL,
    warnings jsonb NOT NULL DEFAULT '[]'::jsonb,
    segment_ids jsonb NOT NULL,
    degraded boolean NOT NULL DEFAULT false,
    generation_duration_ms bigint NOT NULL CHECK (generation_duration_ms >= 0),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX route_candidates_search_score_idx
    ON route_candidates (search_id, total_score DESC);
CREATE UNIQUE INDEX route_candidates_search_rank_idx
    ON route_candidates (search_id, rank)
    WHERE rank IS NOT NULL;
CREATE INDEX route_candidates_geometry_gist_idx
    ON route_candidates USING gist (geometry);

CREATE TABLE search_events (
    search_id uuid NOT NULL REFERENCES route_searches (id) ON DELETE CASCADE,
    sequence bigint NOT NULL CHECK (sequence > 0),
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    occurred_at timestamptz NOT NULL,
    retain_until timestamptz,
    PRIMARY KEY (search_id, sequence)
);

CREATE INDEX search_events_retention_idx
    ON search_events (retain_until)
    WHERE retain_until IS NOT NULL;
