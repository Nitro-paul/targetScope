-- doc/database.md#schema-sql

CREATE TABLE settings (
    id            INTEGER PRIMARY KEY,
    name          TEXT    NOT NULL UNIQUE,
    display_name  TEXT    NOT NULL,
    active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    value         TEXT
);

CREATE TABLE shooters (
    id       INTEGER PRIMARY KEY,
    uuid     TEXT    NOT NULL UNIQUE,
    name     TEXT    NOT NULL UNIQUE COLLATE NOCASE CHECK (name <> '' AND name = trim(name)),
    retired  INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1))
);

CREATE TABLE devices (
    id                  INTEGER PRIMARY KEY,
    uuid                TEXT    NOT NULL UNIQUE,
    name                TEXT    NOT NULL,
    category            TEXT,
    description         TEXT,
    caliber             TEXT,
    image_path          TEXT,
    bullet_diameter_mm  REAL    NOT NULL CHECK (bullet_diameter_mm > 0),
    shot_count          INTEGER NOT NULL DEFAULT 0 CHECK (shot_count >= 0),
    retired             INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1))
);

CREATE TABLE target_types (
    id                     INTEGER PRIMARY KEY,
    uuid                   TEXT    NOT NULL UNIQUE,
    name                   TEXT    NOT NULL UNIQUE COLLATE NOCASE CHECK (name <> '' AND name = trim(name)),
    paper_width_mm         REAL    NOT NULL CHECK (paper_width_mm > 0),
    paper_height_mm        REAL    NOT NULL CHECK (paper_height_mm > 0),
    center_x_mm            REAL    NOT NULL,
    center_y_mm            REAL    NOT NULL,
    inner_white_radius_mm  REAL    NOT NULL CHECK (inner_white_radius_mm > 0),
    inner_white_score      INTEGER NOT NULL,
    black_radius_mm        REAL    NOT NULL,
    black_score            INTEGER NOT NULL,
    outer_radius_mm        REAL    NOT NULL,
    outer_score            INTEGER NOT NULL CHECK (outer_score >= 1),
    retired                INTEGER NOT NULL DEFAULT 0 CHECK (retired IN (0, 1)),
    CHECK (inner_white_radius_mm < black_radius_mm AND black_radius_mm < outer_radius_mm),
    CHECK (inner_white_score > black_score AND black_score > outer_score),
    CHECK (center_x_mm - outer_radius_mm >= 0 AND center_x_mm + outer_radius_mm <= paper_width_mm),
    CHECK (center_y_mm - outer_radius_mm >= 0 AND center_y_mm + outer_radius_mm <= paper_height_mm)
);

CREATE TABLE sessions (
    id              INTEGER PRIMARY KEY,
    uuid            TEXT    NOT NULL UNIQUE,
    saved_at        INTEGER NOT NULL,
    score           INTEGER NOT NULL,
    grouping_score  INTEGER NOT NULL,
    distance_m      INTEGER CHECK (distance_m >= 0),
    target_type_id  INTEGER NOT NULL REFERENCES target_types(id),
    shooter_id      INTEGER REFERENCES shooters(id),
    device_id       INTEGER NOT NULL REFERENCES devices(id),
    single_handed   INTEGER NOT NULL DEFAULT 0 CHECK (single_handed IN (0, 1)),
    supported       INTEGER NOT NULL DEFAULT 0 CHECK (supported IN (0, 1)),
    shot_count      INTEGER NOT NULL CHECK (shot_count >= 1),
    duration_s      INTEGER CHECK (duration_s >= 0),
    notes           TEXT    CHECK (length(notes) <= 10000),
    paper_count     INTEGER NOT NULL CHECK (paper_count >= 1)
);

CREATE TABLE shots (
    id          INTEGER PRIMARY KEY,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    shot_no     INTEGER NOT NULL CHECK (shot_no >= 1),
    paper_no    INTEGER NOT NULL CHECK (paper_no >= 1),
    angle_turns REAL    NOT NULL CHECK (angle_turns >= 0 AND angle_turns < 1),
    radius_mm   REAL    NOT NULL CHECK (radius_mm >= 0),
    UNIQUE (session_id, shot_no)
);

CREATE TABLE session_images (
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    paper_no    INTEGER NOT NULL CHECK (paper_no >= 1),
    image_path  TEXT    NOT NULL,
    PRIMARY KEY (session_id, paper_no)
);

CREATE TABLE processed_requests (
    request_id    TEXT    PRIMARY KEY,
    type          TEXT    NOT NULL,
    processed_at  INTEGER NOT NULL,
    response      TEXT    NOT NULL
);

CREATE INDEX idx_sessions_shooter_id     ON sessions(shooter_id);
CREATE INDEX idx_sessions_device_id      ON sessions(device_id);
CREATE INDEX idx_sessions_target_type_id ON sessions(target_type_id);
CREATE INDEX idx_sessions_saved_at       ON sessions(saved_at);
CREATE INDEX idx_shots_session_id        ON shots(session_id);
CREATE INDEX idx_processed_requests_processed_at ON processed_requests(processed_at);

-- doc/database.md#records
CREATE TRIGGER sessions_are_records BEFORE UPDATE ON sessions
BEGIN SELECT RAISE(ABORT, 'record: sessions cannot be changed'); END;

CREATE TRIGGER shots_are_records BEFORE UPDATE ON shots
BEGIN SELECT RAISE(ABORT, 'record: shots cannot be changed'); END;

CREATE TRIGGER session_images_are_records BEFORE UPDATE ON session_images
BEGIN SELECT RAISE(ABORT, 'record: session images cannot be changed'); END;

-- doc/database.md#records
CREATE TRIGGER target_types_locked_when_used
BEFORE UPDATE OF paper_width_mm, paper_height_mm, center_x_mm, center_y_mm,
                 inner_white_radius_mm, inner_white_score, black_radius_mm, black_score,
                 outer_radius_mm, outer_score, uuid ON target_types
WHEN EXISTS (SELECT 1 FROM sessions WHERE target_type_id = OLD.id)
BEGIN SELECT RAISE(ABORT, 'in_use: target type is used by a session'); END;

CREATE TRIGGER devices_bullet_locked_when_used
BEFORE UPDATE OF bullet_diameter_mm, uuid ON devices
WHEN EXISTS (SELECT 1 FROM sessions WHERE device_id = OLD.id)
BEGIN SELECT RAISE(ABORT, 'in_use: bullet diameter is used by a session'); END;

CREATE TRIGGER devices_shot_count_never_decreases
BEFORE UPDATE OF shot_count ON devices
WHEN NEW.shot_count < OLD.shot_count
BEGIN SELECT RAISE(ABORT, 'record: shot_count cannot decrease'); END;
