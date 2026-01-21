
CREATE TABLE IF NOT EXISTS selfroles (
  userid BIGINT NOT NULL,
  guildid BIGINT NOT NULL,
  roleid BIGINT,
  UNIQUE (userid, guildid)
)

