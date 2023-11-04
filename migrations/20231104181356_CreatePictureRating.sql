-- Add migration script here
CREATE TABLE
  PictureRatings (
    picture_rating_id INTEGER NOT NULL PRIMARY KEY,
    moodboard_id INTEGER NOT NULL,
    rating INTEGER NOT NULL,
    FOREIGN KEY (moodboard_id) REFERENCES Moodboards (moodboard_id)
  );
