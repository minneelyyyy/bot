CREATE TABLE IF NOT EXISTS settings (
  guildid BIGINT NOT NULL PRIMARY KEY,
  positional_role BIGINT,
  banrole BIGINT,
  hoist_selfroles BOOLEAN,
  prefix TEXT
)
