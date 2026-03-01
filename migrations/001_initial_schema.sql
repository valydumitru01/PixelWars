-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- Rounds (each lasts 1 month)
-- ============================================================
CREATE TABLE IF NOT EXISTS rounds (
                                      id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at          TIMESTAMPTZ NOT NULL,
    voting_starts_at TIMESTAMPTZ,
    voting_ends_at   TIMESTAMPTZ,
    is_active        BOOLEAN     NOT NULL DEFAULT true,
    CONSTRAINT only_one_active EXCLUDE USING btree (is_active WITH =) WHERE (is_active = true)
    );

-- ============================================================
-- Users
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
                                     id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    username         VARCHAR(50) UNIQUE NOT NULL,
    email            VARCHAR(255) UNIQUE NOT NULL,
    password_hash    TEXT        NOT NULL,
    is_active        BOOLEAN     NOT NULL DEFAULT true,
    is_disqualified  BOOLEAN     NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_draw_at     TIMESTAMPTZ,
    CONSTRAINT username_min_length CHECK (char_length(username) >= 3),
    CONSTRAINT username_valid_chars CHECK (username ~ '^[a-zA-Z0-9_]+$')
    );

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_last_draw ON users(last_draw_at) WHERE is_disqualified = false;

-- ============================================================
-- Parcels (each user claims 10k contiguous pixels per round)
-- ============================================================
CREATE TABLE IF NOT EXISTS parcels (
                                       id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    round_id    UUID        NOT NULL REFERENCES rounds(id) ON DELETE CASCADE,
    origin_x    INTEGER     NOT NULL CHECK (origin_x >= 0),
    origin_y    INTEGER     NOT NULL CHECK (origin_y >= 0),
    width       INTEGER     NOT NULL CHECK (width > 0),
    height      INTEGER     NOT NULL CHECK (height > 0),
    description TEXT        NOT NULL DEFAULT '',
    is_locked   BOOLEAN     NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT parcel_pixel_count      CHECK (width * height = 10000),
    CONSTRAINT parcel_bounds_x         CHECK (origin_x + width  <= 10000),
    CONSTRAINT parcel_bounds_y         CHECK (origin_y + height <= 10000),
    CONSTRAINT one_parcel_per_round    UNIQUE (user_id, round_id)
    );

CREATE INDEX IF NOT EXISTS idx_parcels_round       ON parcels(round_id);
CREATE INDEX IF NOT EXISTS idx_parcels_user_round  ON parcels(user_id, round_id);
CREATE INDEX IF NOT EXISTS idx_parcels_origin      ON parcels(origin_x, origin_y, width, height, round_id);

-- ============================================================
-- Groups (max 10 members, only adjacent parcels)
-- ============================================================
CREATE TABLE IF NOT EXISTS groups (
                                      id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(100) NOT NULL,
    creator_id UUID         NOT NULL REFERENCES users(id),
    round_id   UUID         NOT NULL REFERENCES rounds(id),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT one_group_per_creator_round UNIQUE (creator_id, round_id)
    );

CREATE INDEX IF NOT EXISTS idx_groups_round ON groups(round_id);

CREATE TABLE IF NOT EXISTS group_members (
                                             group_id  UUID        NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id   UUID        NOT NULL REFERENCES users(id)  ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
    );

CREATE INDEX IF NOT EXISTS idx_group_members_user ON group_members(user_id);

CREATE TABLE IF NOT EXISTS group_invites (
                                             id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id    UUID        NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    from_user   UUID        NOT NULL REFERENCES users(id),
    to_user     UUID        NOT NULL REFERENCES users(id),
    status      VARCHAR(20) NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'accepted', 'declined')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT no_duplicate_pending_invite UNIQUE (group_id, to_user)
    );

-- ============================================================
-- Votes (one per user per round, cast for a parcel or group)
-- ============================================================
CREATE TABLE IF NOT EXISTS votes (
                                     id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    round_id   UUID        NOT NULL REFERENCES rounds(id),
    voter_id   UUID        NOT NULL REFERENCES users(id),
    target_id  UUID        NOT NULL, -- parcel_id or group_id
    target_type VARCHAR(10) NOT NULL CHECK (target_type IN ('parcel', 'group')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT one_vote_per_round UNIQUE (round_id, voter_id)
    );

CREATE INDEX IF NOT EXISTS idx_votes_round    ON votes(round_id);
CREATE INDEX IF NOT EXISTS idx_votes_target   ON votes(target_id, round_id);

-- ============================================================
-- Chat messages
-- ============================================================
CREATE TABLE IF NOT EXISTS chat_messages (
                                             id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_type VARCHAR(20) NOT NULL CHECK (channel_type IN ('global', 'group', 'whisper')),
    channel_id   UUID,       -- NULL for global, group_id for group, recipient_id for whisper
    sender_id    UUID        NOT NULL REFERENCES users(id),
    sender_name  VARCHAR(50) NOT NULL,
    content      TEXT        NOT NULL CHECK (char_length(content) BETWEEN 1 AND 2000),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

CREATE INDEX IF NOT EXISTS idx_chat_global   ON chat_messages(created_at DESC) WHERE channel_type = 'global';
CREATE INDEX IF NOT EXISTS idx_chat_channel  ON chat_messages(channel_id, created_at DESC) WHERE channel_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_chat_whisper  ON chat_messages(channel_id, sender_id, created_at DESC) WHERE channel_type = 'whisper';