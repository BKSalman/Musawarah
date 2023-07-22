-- Your SQL goes here
CREATE TABLE IF NOT EXISTS email_verifications (
  id UUID PRIMARY KEY,
  email TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  user_id UUID NOT NULL,

  FOREIGN KEY(user_id)
    REFERENCES users(id)
    ON DELETE CASCADE
    ON UPDATE CASCADE
);
