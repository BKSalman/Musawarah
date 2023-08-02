-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chapter_comments_mapping (
    parent_comment_id UUID NOT NULL,
    child_comment_id UUID NOT NULL,

    FOREIGN KEY(parent_comment_id)
        REFERENCES chapter_comments(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(child_comment_id)
        REFERENCES chapter_comments(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY(parent_comment_id, child_comment_id)
);
