USE dbminus_test;

CREATE TABLE users (
  id BIGINT AUTO_INCREMENT PRIMARY KEY,
  email VARCHAR(255) NOT NULL,
  full_name VARCHAR(255),
  is_active TINYINT(1) NOT NULL DEFAULT 1,
  balance DECIMAL(12,2) NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SET SESSION cte_max_recursion_depth = 2000;

INSERT INTO users (email, full_name, is_active, balance)
WITH RECURSIVE seq(g) AS (
  SELECT 1 UNION ALL SELECT g + 1 FROM seq WHERE g < 1500
)
SELECT CONCAT('user', g, '@example.com'), CONCAT('User ', g), g % 7 <> 0, (g * 13 % 10000) / 100
FROM seq;

CREATE TABLE app_log (
  logged_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  level VARCHAR(16) NOT NULL,
  message TEXT NOT NULL
);

INSERT INTO app_log (level, message)
WITH RECURSIVE seq(g) AS (
  SELECT 1 UNION ALL SELECT g + 1 FROM seq WHERE g < 40
)
SELECT CASE g % 3 WHEN 0 THEN 'info' WHEN 1 THEN 'warn' ELSE 'error' END, CONCAT('log entry ', g)
FROM seq;

CREATE TABLE types_demo (
  id CHAR(36) PRIMARY KEY,
  payload JSON,
  raw BLOB,
  ratio DOUBLE,
  born DATE,
  wake TIME
);

INSERT INTO types_demo VALUES
  (UUID(), '{"a": 1, "b": [true, null]}', X'DEADBEEF', 3.14, '1990-05-04', '07:30:00'),
  (UUID(), NULL, NULL, NULL, NULL, NULL);