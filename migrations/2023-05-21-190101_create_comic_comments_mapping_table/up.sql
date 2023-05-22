-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_comments_mapping (
    parent_comment_id UUID NOT NULL,
    child_comment_id UUID NOT NULL,

    PRIMARY KEY(parent_comment_id, child_comment_id)
);