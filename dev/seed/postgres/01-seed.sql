CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL,
  full_name TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  balance NUMERIC(12,2) NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO users (email, full_name, is_active, balance)
SELECT
  'user' || g || '@example.com',
  'User ' || g,
  g % 7 <> 0,
  (g * 13 % 10000)::numeric / 100
FROM generate_series(1, 1500) g;

CREATE TABLE app_log (
  logged_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  level TEXT NOT NULL,
  message TEXT NOT NULL
);

INSERT INTO app_log (level, message)
SELECT
  CASE g % 3 WHEN 0 THEN 'info' WHEN 1 THEN 'warn' ELSE 'error' END,
  'log entry ' || g
FROM generate_series(1, 40) g;

CREATE TABLE types_demo (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  payload JSONB,
  raw BYTEA,
  ratio DOUBLE PRECISION,
  born DATE,
  wake TIME
);

INSERT INTO types_demo (payload, raw, ratio, born, wake) VALUES
  ('{"a": 1, "b": [true, null]}', '\xdeadbeef', 3.14, '1990-05-04', '07:30:00'),
  (NULL, NULL, NULL, NULL, NULL);

CREATE VIEW active_users AS SELECT id, email FROM users WHERE is_active;