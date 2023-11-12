-- Add migration script here
CREATE TABLE
  Pictures (
    pictures_id INTEGER NOT NULL PRIMARY KEY,
    moodboard_id INTEGER NOT NULL,
    pexels_id INTEGER NOT NULL,
    url VARCHAR(255) NOT NULL,
    FOREIGN KEY (moodboard_id) REFERENCES Moodboards (moodboard_id)
  );
