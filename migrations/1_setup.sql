CREATE TABLE IF NOT EXISTS images (
        id INTEGER PRIMARY KEY,
        avglf0 REAL NOT NULL,
        avglf1 REAL NOT NULL,
        avglf2 REAL NOT NULL,
        sig0 BLOB,
        sig1 BLOB,
        sig2 BLOB
);