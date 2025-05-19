CREATE TABLE greeting (
  id SERIAL PRIMARY KEY,

  guild BIGINT NOT NULL,
  channel BIGINT,
  enabled BOOLEAN NOT NULL DEFAULT FALSE,

  content VARCHAR(2000),
  mention BOOLEAN NOT NULL DEFAULT FALSE,

  FOREIGN KEY (guild) REFERENCES guild(id) ON DELETE CASCADE,
  UNIQUE (guild)
);

CREATE TABLE greet_embed (
  id SERIAL PRIMARY KEY,
  greeting INT NOT NULL REFERENCES greeting(id) ON DELETE CASCADE,

  thumbnail_image_url VARCHAR(500),
  image_url VARCHAR(500),
  color VARCHAR(10),

  author VARCHAR(256),
  author_icon_url VARCHAR(500),

  description VARCHAR(2000),

  footer VARCHAR(256),
  footer_icon_url VARCHAR(500),
  UNIQUE (greeting)
);
