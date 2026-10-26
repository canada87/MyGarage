PRAGMA foreign_keys = ON;

CREATE TABLE vehicles (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    name              TEXT    NOT NULL,
    make              TEXT,
    model             TEXT,
    plate             TEXT,
    vin               TEXT,
    year              INTEGER,
    registration_date TEXT,
    purchase_date     TEXT,
    purchased_used    INTEGER NOT NULL DEFAULT 0,
    initial_km        INTEGER NOT NULL DEFAULT 0,
    color             TEXT    NOT NULL DEFAULT '#2563EB',
    notes             TEXT,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Categories: assicurazione | bollo | revisione | manutenzione | gomme | abbonamento | altro
CREATE TABLE task_types (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    category        TEXT    NOT NULL DEFAULT 'altro',
    icon            TEXT    NOT NULL DEFAULT 'wrench',
    interval_months INTEGER,
    interval_km     INTEGER,
    -- First occurrence counted from registration date (e.g. revisione: 48 months)
    first_interval_months INTEGER,
    -- Yearly deadline on a fixed day (e.g. winter tyres by 15/11)
    anchor_month    INTEGER,
    anchor_day      INTEGER,
    suggested       INTEGER NOT NULL DEFAULT 1,
    sort_order      INTEGER NOT NULL DEFAULT 100
);

CREATE TABLE schedules (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id       INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    task_type_id     INTEGER REFERENCES task_types(id) ON DELETE SET NULL,
    title            TEXT    NOT NULL,
    category         TEXT    NOT NULL DEFAULT 'altro',
    icon             TEXT    NOT NULL DEFAULT 'wrench',
    interval_months  INTEGER,
    interval_km      INTEGER,
    anchor_month     INTEGER,
    anchor_day       INTEGER,
    -- "Last done" known when the schedule was set up (before any event in the app)
    baseline_date    TEXT,
    baseline_km      INTEGER,
    -- Next due typed by hand; valid until an event dated on/after manual_set_on
    manual_due_date  TEXT,
    manual_due_km    INTEGER,
    manual_set_on    TEXT,
    -- Cached result of the due computation
    last_done_date   TEXT,
    last_done_km     INTEGER,
    next_due_date    TEXT,
    next_due_km      INTEGER,
    warn_days        INTEGER NOT NULL DEFAULT 30,
    warn_km          INTEGER NOT NULL DEFAULT 2000,
    email_enabled    INTEGER NOT NULL DEFAULT 1,
    active           INTEGER NOT NULL DEFAULT 1,
    notes            TEXT,
    created_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_schedules_vehicle ON schedules(vehicle_id);

CREATE TABLE events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id  INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    schedule_id INTEGER REFERENCES schedules(id) ON DELETE SET NULL,
    title       TEXT    NOT NULL,
    category    TEXT    NOT NULL DEFAULT 'altro',
    date        TEXT    NOT NULL,
    km          INTEGER,
    cost_cents  INTEGER,
    provider    TEXT,
    notes       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_events_vehicle ON events(vehicle_id, date);
CREATE INDEX idx_events_schedule ON events(schedule_id, date);

CREATE TABLE odometer_readings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    event_id   INTEGER REFERENCES events(id) ON DELETE CASCADE,
    date       TEXT    NOT NULL,
    km         INTEGER NOT NULL,
    source     TEXT    NOT NULL DEFAULT 'manual',
    note       TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_odometer_vehicle ON odometer_readings(vehicle_id, date);

CREATE TABLE documents (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id    INTEGER REFERENCES vehicles(id) ON DELETE CASCADE,
    event_id      INTEGER REFERENCES events(id) ON DELETE SET NULL,
    title         TEXT    NOT NULL,
    kind          TEXT    NOT NULL DEFAULT 'altro',
    stored_name   TEXT    NOT NULL,
    original_name TEXT    NOT NULL,
    mime          TEXT    NOT NULL,
    size          INTEGER NOT NULL,
    uploaded_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE contacts (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id INTEGER REFERENCES vehicles(id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    category   TEXT    NOT NULL DEFAULT 'altro',
    phone      TEXT,
    email      TEXT,
    reference  TEXT,
    notes      TEXT
);

CREATE TABLE tyre_sets (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_id    INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    name          TEXT    NOT NULL,
    season        TEXT,
    brand         TEXT,
    model         TEXT,
    size          TEXT,
    dot           TEXT,
    purchase_date TEXT,
    initial_km    INTEGER,
    notes         TEXT,
    retired       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tyre_mounts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tyre_set_id INTEGER NOT NULL REFERENCES tyre_sets(id) ON DELETE CASCADE,
    mounted_on  TEXT    NOT NULL,
    mounted_km  INTEGER,
    removed_on  TEXT,
    removed_km  INTEGER
);

CREATE TABLE notification_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL REFERENCES schedules(id) ON DELETE CASCADE,
    kind        TEXT    NOT NULL,
    due_ref     TEXT    NOT NULL,
    sent_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_notification_schedule ON notification_log(schedule_id, kind, due_ref);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO task_types (name, category, icon, interval_months, interval_km, first_interval_months, anchor_month, anchor_day, suggested, sort_order) VALUES
    ('Revisione',               'revisione',    'clipboard', 24,   NULL,   48,   NULL, NULL, 1, 10),
    ('RCA',                     'assicurazione','shield',    12,   NULL,   NULL, NULL, NULL, 1, 20),
    ('Bollo',                   'bollo',        'receipt',   12,   NULL,   NULL, NULL, NULL, 1, 30),
    ('Tagliando',               'manutenzione', 'wrench',    12,   15000,  NULL, NULL, NULL, 1, 40),
    ('Batteria',                'manutenzione', 'battery',   36,   NULL,   NULL, NULL, NULL, 1, 50),
    ('Gomme invernali',         'gomme',        'snow',      NULL, NULL,   NULL, 11,   15,   1, 60),
    ('Gomme estive',            'gomme',        'sun',       NULL, NULL,   NULL, 4,    15,   1, 70),
    ('Freni',                   'manutenzione', 'disc',      NULL, 40000,  NULL, NULL, NULL, 0, 80),
    ('Cinghia distribuzione',   'manutenzione', 'cog',       60,   120000, NULL, NULL, NULL, 0, 90),
    ('Ricarica climatizzatore', 'manutenzione', 'fan',       24,   NULL,   NULL, NULL, NULL, 0, 100),
    ('Soccorso stradale',       'abbonamento',  'truck',     12,   NULL,   NULL, NULL, NULL, 0, 110);
