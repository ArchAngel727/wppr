-- Add migration script here
CREATE TABLE local_images (
  name TEXT NOT NULL,
  timestamp DATETIME NOT NULL UNIQUE
);
