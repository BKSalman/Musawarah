-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_genres_mapping (
    comic_id UUID NOT NULL,
    genre_id SERIAL NOT NULL,

    PRIMARY KEY(comic_id, genre_id),

    FOREIGN KEY(comic_id)
        REFERENCES comics(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(genre_id)
        REFERENCES comic_genres(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);