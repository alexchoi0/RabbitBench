CREATE TABLE projects (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    public BOOLEAN NOT NULL DEFAULT false,
    github_repo TEXT,
    github_token TEXT,
    github_pr_comments BOOLEAN NOT NULL DEFAULT false,
    github_status_checks BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE api_tokens (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE branches (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE testbeds (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE measures (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    name TEXT NOT NULL,
    units TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE benchmarks (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE reports (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    testbed_id UUID NOT NULL,
    hash TEXT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE metrics (
    id UUID PRIMARY KEY,
    report_id UUID NOT NULL,
    benchmark_id UUID NOT NULL,
    measure_id UUID NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE thresholds (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    measure_id UUID NOT NULL,
    branch_id UUID,
    testbed_id UUID,
    upper_boundary DOUBLE PRECISION,
    lower_boundary DOUBLE PRECISION,
    min_sample_size INTEGER NOT NULL DEFAULT 2,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE alerts (
    id UUID PRIMARY KEY,
    threshold_id UUID NOT NULL,
    metric_id UUID NOT NULL,
    boundary TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE flamegraphs (
    id UUID PRIMARY KEY,
    metric_id UUID NOT NULL,
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
