-- PixelWar Initial Schema
-- Run with: sqlx migrate run

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    is_disqualified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_draw_at TIMESTAMPTZ
);

-- Rounds
CREATE TABLE rounds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at TIMESTAMPTZ NOT NULL,
    voting_starts_at TIMESTAMPTZ,
    voting_ends_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true
);

-- Parcels (10k contiguous pixels per user)
CREATE TABLE parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    round_id UUID NOT NULL REFERENCES rounds(id),
    origin_x INTEGER NOT NULL,
    origin_y INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_locked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_parcel_size CHECK (width * height = 10000),
    CONSTRAINT within_canvas CHECK (
        origin_x >= 0 AND origin_y >= 0
        AND origin_x + width <= 10000
        AND origin_y + height <= 10000
    )
);

CREATE INDEX idx_parcels_round ON parcels(round_id);
CREATE INDEX idx_parcels_user ON parcels(user_id);
CREATE INDEX idx_parcels_position ON parcels(origin_x, origin_y, width, height);

-- Groups (max 10 members)
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    creator_id UUID NOT NULL REFERENCES users(id),
    round_id UUID NOT NULL REFERENCES rounds(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE group_invites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    from_user_id UUID NOT NULL REFERENCES users(id),
    to_user_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Votes
CREATE TABLE votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    round_id UUID NOT NULL REFERENCES rounds(id),
    voter_id UUID NOT NULL REFERENCES users(id),
    target_id UUID NOT NULL, -- parcel_id or group_id
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (round_id, voter_id) -- one vote per user per round
);

CREATE INDEX idx_votes_round ON votes(round_id);
CREATE INDEX idx_votes_target ON votes(target_id);

-- Chat messages (recent messages; older ones archived)
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    channel_type VARCHAR(20) NOT NULL, -- 'global', 'group', 'whisper'
    channel_id VARCHAR(255), -- group_id or user_id for whisper
    sender_id UUID NOT NULL REFERENCES users(id),
    sender_name VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_chat_channel ON chat_messages(channel_type, channel_id, created_at DESC);
