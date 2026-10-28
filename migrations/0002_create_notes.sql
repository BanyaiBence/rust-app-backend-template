-- Example resource owned by a user. Every query filters on user_id, so a user can only ever
-- see or change their own notes.
CREATE TABLE notes (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Serves the list query: one user's notes, newest first.
CREATE INDEX notes_user_id_created_at_idx ON notes (user_id, created_at DESC);
