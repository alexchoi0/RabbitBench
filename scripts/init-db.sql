-- Driftwatch App Tables
-- Run after Prisma creates auth tables

CREATE TABLE IF NOT EXISTS api_tokens (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  token_hash VARCHAR(255) NOT NULL,
  last_used_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON api_tokens(token_hash);

CREATE TABLE IF NOT EXISTS projects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  slug VARCHAR(255) NOT NULL,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  public BOOLEAN NOT NULL DEFAULT false,
  github_repo VARCHAR(255),
  github_token TEXT,
  github_pr_comments BOOLEAN NOT NULL DEFAULT false,
  github_status_checks BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id, slug)
);
CREATE INDEX IF NOT EXISTS idx_projects_user_id ON projects(user_id);
CREATE INDEX IF NOT EXISTS idx_projects_github_repo ON projects(github_repo);

CREATE TABLE IF NOT EXISTS branches (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, name)
);
CREATE INDEX IF NOT EXISTS idx_branches_project_id ON branches(project_id);

CREATE TABLE IF NOT EXISTS testbeds (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, name)
);
CREATE INDEX IF NOT EXISTS idx_testbeds_project_id ON testbeds(project_id);

CREATE TABLE IF NOT EXISTS measures (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  units VARCHAR(50),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, name)
);
CREATE INDEX IF NOT EXISTS idx_measures_project_id ON measures(project_id);

CREATE TABLE IF NOT EXISTS benchmarks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name VARCHAR(1024) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, name)
);
CREATE INDEX IF NOT EXISTS idx_benchmarks_project_id ON benchmarks(project_id);

CREATE TABLE IF NOT EXISTS reports (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  branch_id UUID NOT NULL REFERENCES branches(id),
  testbed_id UUID NOT NULL REFERENCES testbeds(id),
  git_hash VARCHAR(40),
  pr_number INTEGER,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_reports_project_id ON reports(project_id);
CREATE INDEX IF NOT EXISTS idx_reports_branch_id ON reports(branch_id);
CREATE INDEX IF NOT EXISTS idx_reports_created_at ON reports(created_at);
CREATE INDEX IF NOT EXISTS idx_reports_pr_number ON reports(pr_number);

CREATE TABLE IF NOT EXISTS metrics (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
  benchmark_id UUID NOT NULL REFERENCES benchmarks(id),
  measure_id UUID NOT NULL REFERENCES measures(id),
  value DOUBLE PRECISION NOT NULL,
  lower_value DOUBLE PRECISION,
  upper_value DOUBLE PRECISION,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_metrics_report_id ON metrics(report_id);
CREATE INDEX IF NOT EXISTS idx_metrics_benchmark_id ON metrics(benchmark_id);
CREATE INDEX IF NOT EXISTS idx_metrics_created_at ON metrics(created_at);

CREATE TABLE IF NOT EXISTS thresholds (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  branch_id UUID REFERENCES branches(id),
  testbed_id UUID REFERENCES testbeds(id),
  measure_id UUID NOT NULL REFERENCES measures(id),
  upper_boundary DOUBLE PRECISION,
  lower_boundary DOUBLE PRECISION,
  min_sample_size INTEGER NOT NULL DEFAULT 2,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_thresholds_project_id ON thresholds(project_id);

DO $$ BEGIN
  CREATE TYPE alert_status AS ENUM ('active', 'acknowledged', 'resolved');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS alerts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  threshold_id UUID NOT NULL REFERENCES thresholds(id) ON DELETE CASCADE,
  metric_id UUID NOT NULL REFERENCES metrics(id) ON DELETE CASCADE,
  baseline_value DOUBLE PRECISION NOT NULL,
  current_value DOUBLE PRECISION NOT NULL,
  percent_change DOUBLE PRECISION NOT NULL,
  status alert_status NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_alerts_threshold_id ON alerts(threshold_id);
CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);

CREATE TABLE IF NOT EXISTS flamegraphs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
  benchmark_id UUID REFERENCES benchmarks(id),
  storage_path VARCHAR(512) NOT NULL,
  file_name VARCHAR(255) NOT NULL,
  file_size INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_flamegraphs_report_id ON flamegraphs(report_id);
